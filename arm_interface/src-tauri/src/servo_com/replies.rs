use com::client::Reply;
use serde::Deserialize;

/// Reply to the push into pose buffer command.
#[derive(Deserialize)]
pub struct PushIntoPoseBufferReply {}

impl Reply for PushIntoPoseBufferReply {}

/// Reply to the clear pose buffer command.
#[derive(Deserialize)]
pub struct ClearPoseBufferReply {}

impl Reply for ClearPoseBufferReply {}

/// Reply to the get pose buffer capacity command.
#[derive(Deserialize)]
pub struct GetPoseBufferCapacityReply {
    pub capacity: usize,
}

impl Reply for GetPoseBufferCapacityReply {}

/// Reply to the get pose buffer available space command.
#[derive(Deserialize)]
pub struct GetPoseBufferAvailableSpaceReply {
    pub available: usize,
}

impl Reply for GetPoseBufferAvailableSpaceReply {}
