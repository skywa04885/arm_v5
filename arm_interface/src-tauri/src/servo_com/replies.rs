use serde::Deserialize;

/// Reply to the push into pose buffer command.
#[derive(Deserialize)]
pub struct PushIntoPoseBufferResponse {}

/// Reply to the clear pose buffer command.
#[derive(Deserialize)]
pub struct ClearPoseBufferResponse {}

/// Reply to the get pose buffer capacity command.
#[derive(Deserialize)]
pub struct GetPoseBufferCapacityResponse {
    pub capacity: usize,
}

/// Reply to the get pose buffer available space command.
#[derive(Deserialize)]
pub struct GetPoseBufferAvailableSpaceResponse {
    pub available: usize,
}
