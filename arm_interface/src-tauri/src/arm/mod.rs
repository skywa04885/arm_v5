use std::sync::Arc;

use kinematics::{
    inverse::solvers::KinematicSolver,
    model::{KinematicParameters, KinematicState},
};

pub mod motion;

pub struct Arm {
    kinematic_parameters: KinematicParameters,
    kinematic_state: KinematicState,
    kinematic_solver: Arc<dyn KinematicSolver>,
}

impl Arm {
    pub fn new(
        kinematic_parameters: KinematicParameters,
        kinematic_state: KinematicState,
        kinematic_solver: Arc<dyn KinematicSolver>,
    ) -> Self {
        Self {
            kinematic_parameters,
            kinematic_state,
            kinematic_solver,
        }
    }

    #[inline]
    pub fn kinematic_parameters(&self) -> &KinematicParameters {
        &self.kinematic_parameters
    }

    #[inline]
    pub fn kinematic_state(&self) -> &KinematicState {
        &self.kinematic_state
    }

    #[inline]
    pub fn kinematic_solver(&self) -> &Arc<dyn KinematicSolver> {
        &self.kinematic_solver
    }
}
