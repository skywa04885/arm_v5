use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use kinematics::model::{KinematicParameters, KinematicState};

/// This response contains the current kinematic state.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetKinematicStateResponse {
    pub kinematic_state: KinematicState,
}

/// This response contains the current kinematic parameters.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetKinematicParametersResponse {
    pub kinematic_parameters: KinematicParameters,
}

/// This command will be sent to update the kinematic state directly.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKinematicStateCommand {
    pub new_kinematic_state: KinematicState,
}

/// This command will move the end effector.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveEndEffectorCommand {
    pub target_position: Vector3<f64>,
}

/// This is the
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveEndEffectorResponse {
    Unreachable,
    Reached {
        delta_position_magnitude: f64,
        iterations: usize,
    },
}

/// This command contains the response to the get vertices command.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetVerticesResponse {
    pub vertices: [Vector3<f64>; 6],
}

