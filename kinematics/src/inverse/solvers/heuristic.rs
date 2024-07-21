use std::sync::Arc;

use nalgebra::Vector3;

use crate::{
    error::KinematicError, forward::algorithms::ForwardKinematicAlgorithm, inverse::algorithms::InverseKinematicAlgorithm, model::{KinematicParameters, KinematicState}
};

use super::{IKSolverResult, KinematicSolver};

pub struct HeuristicSolverBuilder {
    inverse_algorithm: Arc<dyn InverseKinematicAlgorithm>,
    forward_algorithm: Arc<dyn ForwardKinematicAlgorithm>,
    threshold: f64,
    max_iterations: usize,
}

impl HeuristicSolverBuilder {
    pub fn new(
        inverse_algorithm: Arc<dyn InverseKinematicAlgorithm>,
        forward_algorithm: Arc<dyn ForwardKinematicAlgorithm>,
    ) -> Self {
        let threshold: f64 = 0.01;
        let max_iterations: usize = 200_usize;

        Self {
            inverse_algorithm,
            forward_algorithm,
            threshold,
            max_iterations,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;

        self
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;

        self
    }

    pub fn build(self) -> HeuristicSolver {
        HeuristicSolver::new(
            self.inverse_algorithm,
            self.forward_algorithm,
            self.threshold,
            self.max_iterations,
        )
    }
}

pub struct HeuristicSolver {
    inverse_algorithm: Arc<dyn InverseKinematicAlgorithm>,
    forward_algorithm: Arc<dyn ForwardKinematicAlgorithm>,
    threshold: f64,
    max_iterations: usize,
}

impl HeuristicSolver {
    pub fn new(
        inverse_algorithm: Arc<dyn InverseKinematicAlgorithm>,
        forward_algorithm: Arc<dyn ForwardKinematicAlgorithm>,
        threshold: f64,
        max_iterations: usize,
    ) -> Self {
        Self {
            inverse_algorithm,
            forward_algorithm,
            threshold,
            max_iterations,
        }
    }

    pub fn builder(
        inverse_algorithm: Arc<dyn InverseKinematicAlgorithm>,
        forward_algorithm: Arc<dyn ForwardKinematicAlgorithm>,
    ) -> HeuristicSolverBuilder {
        HeuristicSolverBuilder::new(inverse_algorithm, forward_algorithm)
    }
}

impl KinematicSolver for HeuristicSolver {
    fn translate_limb4_end_effector(
        &self,
        params: &KinematicParameters,
        state: &KinematicState,
        target_position: &Vector3<f64>,
    ) -> Result<IKSolverResult, KinematicError> {
        let mut iterations: usize = 0_usize;

        // We need a new kinematic state, since it will be modified during
        //  the solving process.
        let mut new_state: KinematicState = state.clone();

        while iterations < self.max_iterations {
            // Compute the current position using the forward kinematic algorithm.
            let current_position: Vector3<f64> =
                self.forward_algorithm.limb4_position_vector(params, &new_state);

            // Compute the difference between the current and target position, to
            //  know where we should move.
            let delta_position: Vector3<f64> = target_position - current_position;

            // If the magnitude of the delta position is lower than the threshold,
            //  the simply just exit, we've reached the target.
            let delta_position_magnitude = delta_position.magnitude();
            if delta_position_magnitude < self.threshold {
                return Ok(IKSolverResult::Reached {
                    iterations,
                    delta_position_magnitude,
                    new_state,
                });
            }

            // Adjust the new state.
            new_state = self.inverse_algorithm.translate_limb4_end_effector(
                params,
                &new_state,
                &delta_position,
            )?;

            // Increase the iter variable.
            iterations += 1_usize;
        }

        Ok(IKSolverResult::Unreachable)
    }

    fn rotate_limb4_end_effector(
        &self,
        _params: &KinematicParameters,
        _state: &KinematicState,
        _target_position: &Vector3<f64>,
    ) -> Result<IKSolverResult, KinematicError> {
        Ok(IKSolverResult::Unreachable)
    }

    fn inverse_algorithm(&self) -> &Arc<dyn InverseKinematicAlgorithm> {
        &self.inverse_algorithm
    }

    fn forward_algorithm(&self) -> &Arc<dyn ForwardKinematicAlgorithm> {
        &self.forward_algorithm
    }
}
