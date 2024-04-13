use nalgebra::Vector3;
use serde::Serialize;

use crate::kinematics::model::KinematicState;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArmStateChangedEvent {
    pub kinematic_state: KinematicState,
    pub vertices: [Vector3<f64>; 6],
}