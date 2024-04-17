// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, sync::Arc, time::Duration};

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
        analytical::AnalyticalForwardKinematicAlgorithm, compute_arm_vertices,
        ForwardKinematicAlgorithm,
    },
    inverse::{
        algorithms::{heuristic::HeuristicInverseKinematicAlgorithm, InverseKinematicAlgorithm},
        solvers::{
            heuristic::HeuristicSolverBuilder, KinematicInverseSolverResult, KinematicSolver,
        },
    },
    model::{KinematicParameters, KinematicState}, motion::player::{Configuration, Player},
};
use nalgebra::Vector3;
use tauri::Manager;
use tokio::{
    spawn,
    sync::watch::{Receiver as WatchReceiver, Sender as WatchSender},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod frontend;
mod kinematics;
mod servo_com;
mod error;

struct ArmState {
    pub kinematic_parameters: KinematicParameters,
    pub kinematic_state: WatchSender<KinematicState>,
    pub kinematic_solver: Arc<dyn KinematicSolver>,
}

impl Default for ArmState {
    fn default() -> Self {
        // Create the inverse and forward algorithms.
        let inverse_algorithm: Arc<dyn InverseKinematicAlgorithm> =
            Arc::new(HeuristicInverseKinematicAlgorithm::default());
        let forward_algorithm: Arc<dyn ForwardKinematicAlgorithm> =
            Arc::new(AnalyticalForwardKinematicAlgorithm::default());

        // Construct the kinematic solver.
        let kinematic_solver: Arc<dyn KinematicSolver> =
            Arc::new(HeuristicSolverBuilder::new(inverse_algorithm, forward_algorithm).build());

        Self {
            kinematic_parameters: KinematicParameters::default(),
            kinematic_state: WatchSender::new(KinematicState::default()),
            kinematic_solver,
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// This command gets the vertices.
#[tauri::command]
fn get_vertices(arm_state: tauri::State<ArmState>) -> GetVerticesResponse {
    // Get the kinematic parameters and the kinematic state.
    let params: KinematicParameters = arm_state.kinematic_parameters.clone();
    let state: KinematicState = arm_state.kinematic_state.borrow().clone();

    // Compute all the vertices.
    let forward_algorithm: &Arc<dyn ForwardKinematicAlgorithm> =
        arm_state.kinematic_solver.forward_algorithm();
    let vertices: [Vector3<f64>; 6] = compute_arm_vertices(forward_algorithm, &params, &state);

    // Send the response.
    GetVerticesResponse { vertices }
}

/// This handler can be used to get the kinematic state.
#[tauri::command]
fn get_kinematic_state(arm_state: tauri::State<ArmState>) -> GetKinematicStateResponse {
    let kinematic_state: KinematicState = arm_state.kinematic_state.borrow().clone();

    GetKinematicStateResponse { kinematic_state }
}

/// This handler can be used to get the kinematic parameters.
#[tauri::command]
fn get_kinematic_parameters(arm_state: tauri::State<ArmState>) -> GetKinematicParametersResponse {
    let kinematic_parameters: KinematicParameters = arm_state.kinematic_parameters.clone();

    GetKinematicParametersResponse {
        kinematic_parameters,
    }
}

#[tauri::command]
fn move_end_effector(
    arm_state: tauri::State<ArmState>,
    command: MoveEndEffectorCommand,
) -> Result<MoveEndEffectorResponse, String> {
    // Get the kinematic parameters and state.
    let params: KinematicParameters = arm_state.kinematic_parameters.clone();
    let state: KinematicState = arm_state.kinematic_state.borrow().clone();

    // Comoute the new kinematic state.
    let solver_result: KinematicInverseSolverResult = arm_state
        .kinematic_solver
        .translate_limb4_end_effector(&params, &state, &command.target_position)
        .map_err(|_| "Failed to translate end effector")?;

    match solver_result {
        KinematicInverseSolverResult::Reached {
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
        KinematicInverseSolverResult::Unreachable => Ok(MoveEndEffectorResponse::Unreachable),
    }
}

/// This function will handle arm state changes.
async fn handle_arm_state_changes(app_handle: tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let arm_state = app_handle.state::<ArmState>();

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

    // let motion_player_configuration = Configuration::new(0.1_f64);
    // let (mut player_worker, player_handle) = Player::new(client_handle, motion_player_configuration, 
    //     );

    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    // Spawn the client worker.
    task_tracker.spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            client_worker.run(cancellation_token).await.unwrap();
        }
    });

    // Spawn the motion player worker.
    // task_tracker.spawn({
    //     let cancellation_token = cancellation_token.clone();

    //     async move {
    //         player_worker.run(cancellation_token).await.unwrap();
    //     }
    // });

    tauri::Builder::default()
        .manage(ArmState::default())
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
