use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use kinematics::inverse::solvers::{IKSolverResult, KinematicSolver};

use crate::{arm::Arm, error::Error, servo_com::Handle};

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
        handle: Handle,
        configuration: Configuration,
        arm: Arc<Arm>,
    ) -> (Worker, Handle) {
        let (instruction_sender, instruction_receiver) = mpsc::channel(Self::CHANNEL_CAPACITY);

        let worker = Worker::new(handle, instruction_receiver, configuration, arm);
        let handle = Handle::new(instruction_sender);

        (worker, handle)
    }
}

pub(crate) struct Worker {
    handle: Handle,
    instruction_receiver: mpsc::Receiver<Instructon>,
    configuration: Configuration,
    arm: Arc<Arm>,
}

impl Worker {
    pub fn new(
        handle: Handle,
        instruction_receiver: mpsc::Receiver<Instructon>,
        configuration: Configuration,
        arm: Arc<Arm>,
    ) -> Self {
        Self {
            handle,
            instruction_receiver,
            configuration,
            arm,
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

        let mut new_kinematic_state = self.arm.kinematic_state().clone();
        let kinematic_params = self.arm.kinematic_parameters();

        while let Some(target_position) = motion.interpolate(t) {
            new_kinematic_state = match self.arm.kinematic_solver().translate_limb4_end_effector(
                kinematic_params,
                &new_kinematic_state,
                &target_position,
            )? {
                IKSolverResult::Reached { new_state, .. } => new_state,
                IKSolverResult::Unreachable => {
                    return Err(Error::Generic("Could not reach target".into()))
                }
            };

            available -= 1;

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
