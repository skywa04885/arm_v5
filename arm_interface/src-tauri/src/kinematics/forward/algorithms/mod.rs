use std::sync::Arc;

use nalgebra::{Matrix3, Vector3};

use crate::kinematics::model::{KinematicParameters, KinematicState};

pub mod analytical;

pub trait ForwardKinematicAlgorithm: Send + Sync {
    /// Compute the end-effector position of the first limb.
    fn limb0_position_vector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the end-effector position of the second limb.
    fn limb1_position_vector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the end-effector position of the third limb.
    fn limb2_position_vector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the end-effector position of the fourth limb.
    fn limb3_position_vector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the end-effector position of the fifth limb.
    fn limb4_position_vector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the vector of euler angles for the end-effector of the fourth limb.
    fn limb4_euler_angles(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Vector3<f64>;

    /// Compute the orientation matrix of the end-effector of the fourth limb.
    fn limb4_orientation_matrix(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
    ) -> Matrix3<f64>;
}

/// Compute all the vertices of the arm.
pub fn compute_arm_vertices(
    algorithm: &Arc<dyn ForwardKinematicAlgorithm>,
    params: &KinematicParameters,
    state: &KinematicState,
) -> [Vector3<f64>; 6] {
    [
        Vector3::<f64>::zeros(),
        algorithm.limb0_position_vector(params, state),
        algorithm.limb1_position_vector(params, state),
        algorithm.limb2_position_vector(params, state),
        algorithm.limb3_position_vector(params, state),
        algorithm.limb4_position_vector(params, state),
    ]
}
