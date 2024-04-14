use com::{client::Command, proto::CommandCode};
use serde::Serialize;

/// Command that can be sent to push a new pose into the pose buffer.
#[derive(Serialize)]
pub struct PushIntoPoseBufferCommand {
    angles: [f64; 5],
    duration: f64,
}

impl PushIntoPoseBufferCommand {
    pub fn new(angles: [f64; 5], duration: f64) -> Self {
        Self { angles, duration }
    }
}

impl Command for PushIntoPoseBufferCommand {
    /// Get the command code.
    fn code(&self) -> CommandCode {
        CommandCode::new(0x00000100_u32)
    }
}

/// Command that can be sent to clear the pose buffer.
#[derive(Serialize)]
pub struct ClearPoseBufferCommand {}

impl ClearPoseBufferCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for ClearPoseBufferCommand {
    /// Get the command code.
    fn code(&self) -> CommandCode {
        CommandCode::new(0x00000101_u32)
    }
}

/// Command that can be sent to get the capacity of the pose buffer.
#[derive(Serialize)]
pub struct GetPoseBufferCapacityCommand {}

impl GetPoseBufferCapacityCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for GetPoseBufferCapacityCommand {
    /// Get the command code.
    fn code(&self) -> CommandCode {
        CommandCode::new(0x00000102_u32)
    }
}

/// Command that can be sent to get the available space in the pose buffer.
#[derive(Serialize)]
pub struct GetPoseBufferAvailableSpaceCommand {}

impl GetPoseBufferAvailableSpaceCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for GetPoseBufferAvailableSpaceCommand {
    /// Get the command code.
    fn code(&self) -> CommandCode {
        CommandCode::new(0x00000103_u32)
    }
}
