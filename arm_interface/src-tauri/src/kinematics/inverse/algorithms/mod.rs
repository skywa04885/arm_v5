use std::error::Error;
use std::sync::Arc;
use nalgebra::Vector3;

use crate::kinematics::model::{KinematicParameters, KinematicState};

pub mod heuristic;

pub trait InverseKinematicAlgorithm: Send + Sync {
    /// Translate the end-effector position of the fourth link.
    fn translate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        delta: &Vector3<f64>,
    ) -> Result<KinematicState, Arc<dyn Error>>;

    /// Rotate the end-effector of the fourth-link.
    fn rotate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        delta: &Vector3<f64>,
    ) -> Result<KinematicState, Arc<dyn Error>>;
}
