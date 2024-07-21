use std::sync::Arc;

use com::client;
use tokio::sync::{broadcast, Notify};
use tokio_util::sync::CancellationToken;

use crate::{error::Error, servo_com::events::PoseChangedEvent};

use self::{
    commands::{ClearPoseBufferCommand, PushIntoPoseBufferCommand},
    events::{PoseBufferDrainEvent, PoseBufferEmptyEvent},
    replies::{ClearPoseBufferReply, GetPoseBufferCapacityReply, PushIntoPoseBufferReply},
};

pub mod commands;
pub mod events;
pub mod replies;

pub struct Broadcasts {
    pose_changed: broadcast::Sender<PoseChangedEvent>,
}

impl Broadcasts {
    pub fn new() -> Self {
        let (pose_changed, _) = broadcast::channel(1);

        Self { pose_changed }
    }

    pub fn pose_changed(&self) -> &broadcast::Sender<PoseChangedEvent> {
        &self.pose_changed
    }
}

pub struct Notifiers {
    drain: Notify,
    empty: Notify,
}

impl Notifiers {
    pub fn new() -> Self {
        Self {
            drain: Notify::new(),
            empty: Notify::new(),
        }
    }

    pub fn drain(&self) -> &Notify {
        &self.drain
    }

    pub fn empty(&self) -> &Notify {
        &self.empty
    }
}

pub struct Worker {
    notifiers: Arc<Notifiers>,
    broadcasts: Arc<Broadcasts>,
    handle: client::Handle,
}

impl Worker {
    pub(self) async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        // Subscribe to the pose changed event (and handle it).
        let pose_changed_ev_sub = self
            .handle
            .serde_sub_to_ev::<PoseChangedEvent>(PoseChangedEvent::CODE, {
                let broadcasts = self.broadcasts.clone();

                move |x| {
                    if let Ok(event) = x {
                        broadcasts.pose_changed.send(event);
                    }
                }
            })
            .await?;

        // Subscribe to the pose buffer drain event (and handle it).
        let pose_buffer_drain_ev_sub = self
            .handle
            .serde_sub_to_ev::<PoseBufferDrainEvent>(PoseBufferDrainEvent::CODE, {
                let notifiers = self.notifiers.clone();

                move |x| {
                    if let Ok(_) = x {
                        notifiers.drain.notify_waiters();
                    }
                }
            })
            .await?;

        // Subscribe to the pose buffer empty event (and handle it).
        let pose_buffer_empty_ev_sub = self
            .handle
            .serde_sub_to_ev::<PoseBufferEmptyEvent>(PoseBufferEmptyEvent::CODE, {
                let notifiers = self.notifiers.clone();

                move |x| {
                    if let Ok(_) = x {
                        notifiers.empty.notify_waiters();
                    }
                }
            })
            .await?;

        // Wait for the cancellation.
        cancellation_token.cancelled().await;

        // Unsubscribe from the pose changed event.
        self.handle
            .unsub_ev(PoseChangedEvent::CODE, pose_changed_ev_sub)
            .await?;

        // Unsubscribe from the pose buffer drain event.
        self.handle
            .unsub_ev(PoseBufferDrainEvent::CODE, pose_buffer_drain_ev_sub)
            .await?;

        // Unsubscribe from the pose buffer empty event.
        self.handle
            .unsub_ev(PoseBufferEmptyEvent::CODE, pose_buffer_empty_ev_sub)
            .await?;

        Ok(())
    }
}

pub struct Handle {
    notifiers: Notifiers,
    handle: client::Handle,
}

impl Handle {
    pub(crate) fn new(notifiers: Notifiers, handle: client::Handle) -> Self {
        Self { notifiers, handle }
    }

    #[inline]
    pub fn notifiers(&self) -> &Notifiers {
        &self.notifiers
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
            .serde_write_cmd_wc::<_, PushIntoPoseBufferReply>(command, cancellation_token)
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
            .serde_write_cmd_wc(command, &cancellation_token)
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
            .serde_write_cmd_wc::<_, ClearPoseBufferReply>(command, cancellation_token)
            .await?;

        Ok(())
    }
}
