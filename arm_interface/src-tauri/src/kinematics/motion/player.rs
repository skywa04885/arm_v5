use std::time::{Duration, Instant};

use com::client;
use tokio::{select, sync::mpsc, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{
    kinematics::error::Error,
    servo_com::{
        commands::{ClearPoseBufferCommand, GetPoseBufferCapacityCommand},
        replies::{ClearPoseBufferReply, GetPoseBufferCapacityReply},
    },
};

use super::Motion;

pub(crate) struct Configuration {
    delta_time: Duration,
}

impl Configuration {
    pub fn new(delta_time: Duration) -> Self {
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

    pub fn new(client_handle: client::Handle, configuration: Configuration) -> (Worker, Handle) {
        let (instruction_sender, instruction_receiver) = mpsc::channel(Self::CHANNEL_CAPACITY);

        let worker = Worker::new(client_handle, instruction_receiver, configuration);
        let handle = Handle::new(instruction_sender);

        (worker, handle)
    }
}

pub(crate) struct Worker {
    client_handle: client::Handle,
    instruction_receiver: mpsc::Receiver<Instructon>,
    configuration: Configuration,
    motion: Option<Box<dyn Motion>>,
}

impl Worker {
    pub fn new(
        client_handle: client::Handle,
        instruction_receiver: mpsc::Receiver<Instructon>,
        configuration: Configuration,
    ) -> Self {
        Self {
            client_handle,
            instruction_receiver,
            configuration,
            motion: None,
        }
    }

    /// Retrieves the buffer capacity for the task.
    ///
    /// This function sends a command to the client and waits for the response containing the capacity
    /// of the pose buffer. It returns the capacity as a `usize` if successful, or an `Error` if an
    /// error occurs during the process.
    ///
    /// # Arguments
    ///
    /// * `cancellation_token` - A reference to a `CancellationToken` used for cancellation.
    ///
    /// # Returns
    ///
    /// * `Result<usize, Error>` - The buffer capacity if successful, or an `Error` if an error occurs.
    pub(self) async fn get_buffer_capacity(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<usize, Error> {
        let command = ClearPoseBufferCommand::new();

        // Send the command and wait for the response containing the capacity.
        let GetPoseBufferCapacityReply { capacity } = self
            .client_handle
            .write_serializable_command_with_cancellation(command, &cancellation_token)
            .await?;

        // Return the capacity.
        Ok(capacity)
    }

    /// Clears the pose buffer.
    ///
    /// This function sends a command to the client to clear the pose buffer. It returns `Ok(())` if
    /// successful, or an `Error` if an error occurs during the process.
    ///
    /// # Arguments
    ///
    /// * `cancellation_token` - A reference to a `CancellationToken` used for cancellation.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - `Ok(())` if successful, or an `Error` if an error occurs.
    pub(self) async fn clear_pose_buffer(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        let command = ClearPoseBufferCommand::new();

        _ = self
            .client_handle
            .write_serializable_command_with_cancellation::<_, ClearPoseBufferReply>(
                command,
                cancellation_token,
            )
            .await?;

        Ok(())
    }
    pub(crate) async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        let start_instant = Instant::now();

        // Clear the pose buffer, to discard any previous movements.
        self.clear_pose_buffer(&cancellation_token).await?;

        // Get the total capacity of the buffer.
        let capacity = self.get_buffer_capacity(&cancellation_token).await?;

        loop {}
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
