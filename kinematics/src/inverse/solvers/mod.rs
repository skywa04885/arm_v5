use std::sync::Arc;

use nalgebra::Vector3;
use serde::Serialize;

use crate::{
    error::KinematicError,
    forward::algorithms::ForwardKinematicAlgorithm,
    model::{KinematicParameters, KinematicState},
};

use super::algorithms::InverseKinematicAlgorithm;

pub mod heuristic;

#[derive(Serialize)]
pub enum IKSolverResult {
    Unreachable,
    Reached {
        iterations: usize,
        delta_position_magnitude: f64,
        new_state: KinematicState,
    },
}

pub trait KinematicSolver: Send + Sync {
    /// Translate the end-effector position of the fourth link.
    fn translate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        target_position: &Vector3<f64>,
    ) -> Result<IKSolverResult, KinematicError>;

    /// Rotate the end-effector of the fourth-link.
    fn rotate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        target_position: &Vector3<f64>,
    ) -> Result<IKSolverResult, KinematicError>;

    fn inverse_algorithm(&self) -> &Arc<dyn InverseKinematicAlgorithm>;

    fn forward_algorithm(&self) -> &Arc<dyn ForwardKinematicAlgorithm>;
}
