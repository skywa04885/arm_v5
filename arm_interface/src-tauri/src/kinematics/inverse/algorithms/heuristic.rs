use std::error::Error;
use std::sync::Arc;

use nalgebra::{Matrix3x5, Matrix5x3, Vector3, Vector5};
use thiserror::Error;

use crate::kinematics::error::KinematicError;
use crate::kinematics::inverse::algorithms::InverseKinematicAlgorithm;
use crate::kinematics::model::{KinematicParameters, KinematicState};

#[derive(Debug, Error)]
pub enum HeuristicInverseKinematicsAlgorithmError {
    #[error("Failed to pseudo-invert jacobian matrix, error: {0}")]
    PseudoInvertFailure(&'static str),
}

pub struct HeuristicInverseKinematicAlgorithm {
    pseudo_inverse_eps: f64,
}

impl Default for HeuristicInverseKinematicAlgorithm {
    fn default() -> Self {
        Self {
            pseudo_inverse_eps: 0.0000000000001,
        }
    }
}

impl HeuristicInverseKinematicAlgorithm {
    fn limb4_end_effector_position_jacobian(
        &self,
        &KinematicParameters {
            l_0,
            l_1,
            l_2,
            l_3,
            l_4,
        }: &KinematicParameters,
        &KinematicState {
            theta_0,
            theta_1,
            theta_2,
            theta_3,
            theta_4,
        }: &KinematicState,
    ) -> Matrix3x5<f64> {
        Matrix3x5::<f64>::new(
            (l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin()
                + l_4 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.cos(),
            (l_1 * theta_1.cos()
                + l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos()
                + l_4 * (theta_1 + theta_2 + theta_3).cos())
                * theta_0.sin(),
            (l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos()
                + l_4 * (theta_1 + theta_2 + theta_3).cos())
                * theta_0.sin(),
            (l_3 + l_4) * theta_0.sin() * (theta_1 + theta_2 + theta_3).cos(),
            0_f64,
            0_f64,
            -l_1 * theta_1.sin()
                - l_2 * (theta_1 + theta_2).sin()
                - l_3 * (theta_1 + theta_2 + theta_3).sin()
                - l_4 * (theta_1 + theta_2 + theta_3).sin(),
            -l_2 * (theta_1 + theta_2).sin()
                - l_3 * (theta_1 + theta_2 + theta_3).sin()
                - l_4 * (theta_1 + theta_2 + theta_3).sin(),
            (-l_3 - l_4) * (theta_1 + theta_2 + theta_3).sin(),
            0_f64,
            (l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin()
                + l_4 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.sin(),
            -(l_1 * theta_1.cos()
                + l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos()
                + l_4 * (theta_1 + theta_2 + theta_3).cos())
                * theta_0.cos(),
            -(l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos()
                + l_4 * (theta_1 + theta_2 + theta_3).cos())
                * theta_0.cos(),
            (-l_3 - l_4) * theta_0.cos() * (theta_1 + theta_2 + theta_3).cos(),
            0_f64,
        )
    }
}

impl InverseKinematicAlgorithm for HeuristicInverseKinematicAlgorithm {
    fn translate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        delta: &Vector3<f64>,
    ) -> Result<KinematicState, KinematicError> {
        // Compute the jacobian matrix for the end-effector position.
        let jacobian: Matrix3x5<f64> = self.limb4_end_effector_position_jacobian(params, state);

        // Invert the jacobian matrix.
        let jacobian_inverse: Matrix5x3<f64> =
            match jacobian.pseudo_inverse(self.pseudo_inverse_eps) {
                Ok(x) => x,
                Err(error) => {
                    return Err(KinematicError::InversionFailure);
                }
            };


        println!("{:?}", (jacobian));
        // Compute the new kinematic state and return it.
        Ok(KinematicState::from(
            (jacobian_inverse * delta) + Vector5::<f64>::from(state),
        ))
    }

    fn rotate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        delta: &Vector3<f64>,
    ) -> Result<KinematicState, KinematicError> {
        todo!()
    }
}

#[cfg(test)]
pub mod tests {
    use nalgebra::Vector3;
    use crate::kinematics::forward::algorithms::analytical::AnalyticalForwardKinematicAlgorithm;
    use crate::kinematics::forward::algorithms::ForwardKinematicAlgorithm;
    use crate::kinematics::inverse::algorithms::heuristic::HeuristicInverseKinematicAlgorithm;
    use crate::kinematics::inverse::algorithms::InverseKinematicAlgorithm;
    use crate::kinematics::model::{KinematicParameters, KinematicState};

    #[test]
    pub fn solve() {
        // Create the kinematic state (in a way that all links point straight up).
        let mut state: KinematicState = KinematicState {
            theta_0: 0_f64,
            theta_1: 0_f64,
            theta_2: 0_f64,
            theta_3: 0_f64,
            theta_4: 0_f64,
        };

        // Create the default kinematic parameters.
        let params: KinematicParameters = KinematicParameters::default();

        // Create the analytical forward kinematics algorithm and the heuristic
        //  inverse kinematics algorithm.
        let fk_solver: AnalyticalForwardKinematicAlgorithm =
            AnalyticalForwardKinematicAlgorithm::default();
        let ik_solver: HeuristicInverseKinematicAlgorithm =
            HeuristicInverseKinematicAlgorithm::default();

        let thresh: f64 = 10_f64.powf(-4_f64);

        let target: Vector3<f64> = Vector3::<f64>::new(2_f64, 48_f64, 2_f64);

        for _ in 1..100 {
            // Compute the current end effector position, and the difference between it and the
            //  target.
            let delta: Vector3<f64> = target - fk_solver.limb4_position_vector(&params, &state);
            println!("{}", delta.magnitude());

            // If the target is really close, just break.
            if delta.magnitude() < thresh {
                break;
            }

            // Update the state.
            state = ik_solver.translate_limb4_end_effector(&params, &state, &delta).unwrap()
        }

        // Make sure that the algorithm reached the destinaton.
        assert!((fk_solver.limb4_position_vector(&params, &state) - target).magnitude() < thresh);
    }
}
