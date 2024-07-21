// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::Arc};

use arm::{
    motion::player::{self, Player},
    Arm,
};
use com::client::Client;
use frontend::{
    commands::arm::{
        GetKinematicParametersResponse, GetKinematicStateResponse, GetVerticesResponse,
        MoveEndEffectorCommand, MoveEndEffectorResponse,
    },
    events::arm::ArmStateChangedEvent,
};
use kinematics::{
    forward::algorithms::{
        analytical::AnalyticalFKAlgorithm, compute_arm_vertices, ForwardKinematicAlgorithm,
    },
    inverse::{
        algorithms::heuristic::HeuristicIKAlgorithm,
        solvers::{heuristic::HeuristicSolver, IKSolverResult},
    },
    model::{KinematicParameters, KinematicState},
};
use nalgebra::Vector3;
use servo_com::Handle;
use tauri::Manager;
use tokio::sync::watch::Receiver as WatchReceiver;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod arm;
mod error;
mod frontend;
mod servo_com;

struct AppState {
    player_handle: player::Handle,
}

impl AppState {
    pub fn new(player_handle: player::Handle) -> Self {
        Self { player_handle }
    }

    #[inline]
    pub fn player_handle(&self) -> &player::Handle {
        &self.player_handle
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// This command gets the vertices.
#[tauri::command]
fn get_vertices(arm_state: tauri::State<AppState>) -> GetVerticesResponse {
    todo!()
}

/// This handler can be used to get the kinematic state.
#[tauri::command]
fn get_kinematic_state(arm_state: tauri::State<AppState>) -> GetKinematicStateResponse {
    let kinematic_state: KinematicState = arm_state.kinematic_state.borrow().clone();

    GetKinematicStateResponse { kinematic_state }
}

/// This handler can be used to get the kinematic parameters.
#[tauri::command]
fn get_kinematic_parameters(arm_state: tauri::State<AppState>) -> GetKinematicParametersResponse {
    let kinematic_parameters: KinematicParameters = arm_state.kinematic_parameters.clone();

    GetKinematicParametersResponse {
        kinematic_parameters,
    }
}

#[tauri::command]
fn move_end_effector(
    arm_state: tauri::State<AppState>,
    command: MoveEndEffectorCommand,
) -> Result<MoveEndEffectorResponse, String> {
    // Get the kinematic parameters and state.
    let params: KinematicParameters = arm_state.kinematic_parameters.clone();
    let state: KinematicState = arm_state.kinematic_state.borrow().clone();

    // Comoute the new kinematic state.
    let solver_result: IKSolverResult = arm_state
        .kinematic_solver
        .translate_limb4_end_effector(&params, &state, &command.target_position)
        .map_err(|_| "Failed to translate end effector")?;

    match solver_result {
        IKSolverResult::Reached {
            iterations,
            delta_position_magnitude,
            new_state,
        } => {
            // Send the new kinematic state.
            arm_state
                .kinematic_state
                .send(new_state)
                .map_err(|_| "Failed to send new kinematic state")?;

            // Return that we reached the target position.
            Ok(MoveEndEffectorResponse::Reached {
                delta_position_magnitude,
                iterations,
            })
        }
        IKSolverResult::Unreachable => Ok(MoveEndEffectorResponse::Unreachable),
    }
}

/// This function will handle arm state changes.
async fn handle_arm_state_changes(app_handle: tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let arm_state = app_handle.state::<AppState>();

    let mut receiver: WatchReceiver<KinematicState> = arm_state.kinematic_state.subscribe();

    loop {
        // Wait for the kinematic state to be changed.
        receiver.changed().await?;

        // Get the kinematic parameters and the kinematic state.
        let params: KinematicParameters = arm_state.kinematic_parameters.clone();
        let state: KinematicState = receiver.borrow().clone();

        // Compute all the vertices.
        let forward_algorithm: &Arc<dyn ForwardKinematicAlgorithm> =
            arm_state.kinematic_solver.forward_algorithm();
        let vertices: [Vector3<f64>; 6] = compute_arm_vertices(forward_algorithm, &params, &state);

        // Publish the event.
        app_handle.emit_all(
            "arm:state-changed",
            ArmStateChangedEvent {
                kinematic_state: state,
                vertices,
            },
        )?;
    }
}

#[tokio::main]
async fn main() {
    let (client_handle, mut client_worker) = Client::connect("127.0.0.1:5000").await.unwrap();

    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    // Spawn the client worker.
    task_tracker.spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            client_worker.run(cancellation_token).await.unwrap();
        }
    });

    let arm = Arc::new(Arm::new(
        KinematicParameters::default(),
        KinematicState::default(),
        {
            let ik = Arc::new(HeuristicIKAlgorithm::default());
            let fk = Arc::new(AnalyticalFKAlgorithm::default());
            Arc::new(HeuristicSolver::builder(ik, fk).build())
        },
    ));

    let player_configuration = player::Configuration::new(0.05_f64);
    let (player_worker, player_handle) = Player::new(
        Handle::new(client_handle),
        player_configuration,
        arm,
    );

    // Spawn the motion player worker.
    // task_tracker.spawn({
    //     let cancellation_token = cancellation_token.clone();

    //     async move {
    //         player_worker.run(cancellation_token).await.unwrap();
    //     }
    // });

    tauri::Builder::default()
        .manage(AppState::new(player_handle))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_kinematic_state,
            get_kinematic_parameters,
            move_end_effector,
            get_vertices
        ])
        .setup(|app| {
            tauri::async_runtime::spawn({
                let app_handle = app.app_handle();
                async move { handle_arm_state_changes(app_handle).await.unwrap() }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    cancellation_token.cancel();

    task_tracker.close();
    task_tracker.wait().await;
}
