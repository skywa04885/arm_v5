use com::{client::Event, proto::EventCode};
use serde::Deserialize;

/// Represents an event that is emitted when the arm pose changes.
#[derive(Deserialize)]
pub struct PoseChangedEvent {
    pub angles: [f64; 5],
}

impl Event for PoseChangedEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        EventCode::new(0x00000000_u32)
    }
}

/// Represents an event that is emitted when the buffer is partially drained.
#[derive(Deserialize)]
pub struct PoseBufferDrainEvent {
    pub available: usize,
}

impl Event for PoseBufferDrainEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        EventCode::new(0x00000100_u32)
    }
}

/// Represents an event that is emitted when the pose buffer is empty.
#[derive(Deserialize)]
pub struct PoseBufferEmptyEvent {}

impl Event for PoseBufferEmptyEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        EventCode::new(0x00000101_u32)
    }
}
