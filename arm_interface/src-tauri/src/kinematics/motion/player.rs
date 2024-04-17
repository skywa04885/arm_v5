use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    kinematics::{
        inverse::solvers::{KinematicInverseSolverResult, KinematicSolver},
        model::{KinematicParameters, KinematicState},
    },
    servo_com::ServoClientHandleFacade,
};

use super::Motion;

pub(crate) struct Configuration {
    delta_time: f64,
}

impl Configuration {
    pub fn new(delta_time: f64) -> Self {
        Self { delta_time }
    }
}

pub(crate) enum Instructon {
    Start(Box<dyn Motion>),
    Stop,
}

pub(crate) struct Player;

impl Player {
    pub const CHANNEL_CAPACITY: usize = 64_usize;

    pub fn new(
        handle: ServoClientHandleFacade,
        configuration: Configuration,
        kinematic_solver: Box<dyn KinematicSolver>,
        kinematic_params: KinematicParameters,
        kinematic_state: KinematicState,
    ) -> (Worker, Handle) {
        let (instruction_sender, instruction_receiver) = mpsc::channel(Self::CHANNEL_CAPACITY);

        let worker = Worker::new(
            handle,
            instruction_receiver,
            configuration,
            kinematic_solver,
            kinematic_params,
            kinematic_state,
        );
        let handle = Handle::new(instruction_sender);

        (worker, handle)
    }
}

pub(crate) struct Worker {
    handle: ServoClientHandleFacade,
    instruction_receiver: mpsc::Receiver<Instructon>,
    configuration: Configuration,
    kinematic_solver: Box<dyn KinematicSolver>,
    kparams: KinematicParameters,
    kinematic_state: KinematicState,
}

impl Worker {
    pub fn new(
        handle: ServoClientHandleFacade,
        instruction_receiver: mpsc::Receiver<Instructon>,
        configuration: Configuration,
        kinematic_solver: Box<dyn KinematicSolver>,
        kinematic_params: KinematicParameters,
        kinematic_state: KinematicState,
    ) -> Self {
        Self {
            handle,
            instruction_receiver,
            configuration,
            kinematic_solver,
            kparams: kinematic_params,
            kinematic_state,
        }
    }

    async fn run_motion(
        &mut self,
        motion: Box<dyn Motion>,
        cancellation_token: CancellationToken,
    ) -> Result<(), Error> {
        self.handle.clear_pose_buffer(&cancellation_token).await?;

        let mut available = self.handle.get_buffer_capacity(&cancellation_token).await?;

        let mut t = 0_f64;

        let mut new_kstate = self.kinematic_state.clone();

        while let Some(target_position) = motion.interpolate(t) {
            new_kstate = match self.kinematic_solver.translate_limb4_end_effector(
                &self.kparams,
                &new_kstate,
                &target_position,
            )? {
                KinematicInverseSolverResult::Reached {
                    iterations,
                    delta_position_magnitude,
                    new_state,
                } => new_state,
                KinematicInverseSolverResult::Unreachable => {
                    return Err(Error::Generic("Could not reach target".into()))
                }
            };

            // self.handle.

            t += self.configuration.delta_time;
        }

        Ok(())
    }

    pub(crate) async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        todo!()
    }
}

pub(crate) struct Handle {
    instruction_sender: mpsc::Sender<Instructon>,
}

impl Handle {
    pub fn new(instruction_sender: mpsc::Sender<Instructon>) -> Self {
        Self { instruction_sender }
    }
}
