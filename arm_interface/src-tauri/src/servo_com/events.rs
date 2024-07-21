use com::{client::Event, proto::EventCode};
use serde::Deserialize;

/// Represents an event that is emitted when the arm pose changes.
#[derive(Clone, Deserialize)]
pub struct PoseChangedEvent {
    pub angles: [f64; 5],
}

impl PoseChangedEvent {
    pub const CODE: EventCode = EventCode::const_new(0x00000000_u32);
}

impl Event for PoseChangedEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        Self::CODE
    }
}

/// Represents an event that is emitted when the buffer is partially drained.
#[derive(Deserialize)]
pub struct PoseBufferDrainEvent {
    pub available: usize,
}

impl PoseBufferDrainEvent {
    pub const CODE: EventCode = EventCode::const_new(0x00000001_u32);
}

impl Event for PoseBufferDrainEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        Self::CODE
    }
}

/// Represents an event that is emitted when the pose buffer is empty.
#[derive(Deserialize)]
pub struct PoseBufferEmptyEvent {}

impl PoseBufferEmptyEvent {
    pub const CODE: EventCode = EventCode::const_new(0x00000002_u32);
}

impl Event for PoseBufferEmptyEvent {
    /// Get the event code.
    fn code(&self) -> EventCode {
        Self::CODE
    }
}
