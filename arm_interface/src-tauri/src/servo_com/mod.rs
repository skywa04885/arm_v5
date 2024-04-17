use com::client::Handle;
use tokio_util::sync::CancellationToken;

use crate::error::Error;

use self::{
    commands::{ClearPoseBufferCommand, PushIntoPoseBufferCommand},
    replies::{ClearPoseBufferReply, GetPoseBufferCapacityReply, PushIntoPoseBufferReply},
};

pub mod commands;
pub mod events;
pub mod replies;

pub struct ServoClientHandleFacade {
    handle: Handle,
}

impl ServoClientHandleFacade {
    pub(crate) fn new(handle: Handle) -> Self {
        Self { handle }
    }

    pub(crate) async fn push_into_pose_buffer(
        &mut self,
        angles: [f64; 5],
        duration: f64,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        let command = PushIntoPoseBufferCommand::new(angles, duration);

        _ = self
            .handle
            .write_serializable_command_with_cancellation::<_, PushIntoPoseBufferReply>(
                command,
                cancellation_token,
            )
            .await?;

        Ok(())
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
    pub(crate) async fn get_buffer_capacity(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<usize, Error> {
        let command = ClearPoseBufferCommand::new();

        // Send the command and wait for the response containing the capacity.
        let GetPoseBufferCapacityReply { capacity } = self
            .handle
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
    pub(crate) async fn clear_pose_buffer(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        let command = ClearPoseBufferCommand::new();

        _ = self
            .handle
            .write_serializable_command_with_cancellation::<_, ClearPoseBufferReply>(
                command,
                cancellation_token,
            )
            .await?;

        Ok(())
    }
}
