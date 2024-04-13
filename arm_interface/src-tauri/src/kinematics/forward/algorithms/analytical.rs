use nalgebra::{Matrix3, Vector3};

use crate::kinematics::forward::algorithms::ForwardKinematicAlgorithm;
use crate::kinematics::model::{KinematicParameters, KinematicState};

/// Analytical forward kinematic approach, see the derivation notebook for the specifics.
pub struct AnalyticalForwardKinematicAlgorithm {}

impl Default for AnalyticalForwardKinematicAlgorithm {
    fn default() -> Self {
        Self {}
    }
}

impl ForwardKinematicAlgorithm for AnalyticalForwardKinematicAlgorithm {
    fn limb0_position_vector(
        &self,
        &KinematicParameters { l_0, .. }: &KinematicParameters,
        &KinematicState { .. }: &KinematicState,
    ) -> Vector3<f64> {
        Vector3::<f64>::new(0_f64, l_0, 0_f64)
    }

    fn limb1_position_vector(
        &self,
        &KinematicParameters { l_0, l_1, .. }: &KinematicParameters,
        &KinematicState {
            theta_0, theta_1, ..
        }: &KinematicState,
    ) -> Vector3<f64> {
        Vector3::<f64>::new(
            l_1 * theta_0.sin() * theta_1.sin(),
            l_0 + l_1 * theta_1.cos(),
            -l_1 * theta_1.sin() * theta_0.cos(),
        )
    }

    fn limb2_position_vector(
        &self,
        &KinematicParameters { l_0, l_1, l_2, .. }: &KinematicParameters,
        &KinematicState {
            theta_0,
            theta_1,
            theta_2,
            ..
        }: &KinematicState,
    ) -> Vector3<f64> {
        Vector3::<f64>::new(
            (l_1 * theta_1.sin() + l_2 * (theta_1 + theta_2).sin()) * theta_0.sin(),
            l_0 + l_1 * theta_1.cos() + l_2 * (theta_1 + theta_2).cos(),
            -(l_1 * theta_1.sin() + l_2 * (theta_1 + theta_2).sin()) * theta_0.cos(),
        )
    }

    fn limb3_position_vector(
        &self,
        &KinematicParameters {
            l_0, l_1, l_2, l_3, ..
        }: &KinematicParameters,
        &KinematicState {
            theta_0,
            theta_1,
            theta_2,
            theta_3,
            ..
        }: &KinematicState,
    ) -> Vector3<f64> {
        Vector3::<f64>::new(
            (l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.sin(),
            l_0 + l_1 * theta_1.cos()
                + l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos(),
            -(l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.cos(),
        )
    }

    fn limb4_position_vector(
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
    ) -> Vector3<f64> {
        Vector3::<f64>::new(
            (l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin()
                + l_4 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.sin(),
            l_0 + l_1 * theta_1.cos()
                + l_2 * (theta_1 + theta_2).cos()
                + l_3 * (theta_1 + theta_2 + theta_3).cos()
                + l_4 * (theta_1 + theta_2 + theta_3).cos(),
            -(l_1 * theta_1.sin()
                + l_2 * (theta_1 + theta_2).sin()
                + l_3 * (theta_1 + theta_2 + theta_3).sin()
                + l_4 * (theta_1 + theta_2 + theta_3).sin())
                * theta_0.cos(),
        )
    }

    fn limb4_euler_angles(
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
    ) -> Vector3<f64> {
        todo!()
    }

    fn limb4_orientation_matrix(
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
    ) -> Matrix3<f64> {
        Matrix3::<f64>::new(
            -theta_0.sin() * theta_4.sin() * (theta_1 + theta_2 + theta_3).cos()
                + theta_0.cos() * theta_4.cos(),
            theta_0.sin() * (theta_1 + theta_2 + theta_3).sin(),
            -theta_0.sin() * theta_4.cos() * (theta_1 + theta_2 + theta_3).cos()
                - theta_4.sin() * theta_0.cos(),
            theta_4.sin() * (theta_1 + theta_2 + theta_3).sin(),
            (theta_1 + theta_2 + theta_3).cos(),
            (theta_1 + theta_2 + theta_3).sin() * theta_4.cos(),
            theta_0.sin() * theta_4.cos()
                + theta_4.sin() * theta_0.cos() * (theta_1 + theta_2 + theta_3).cos(),
            -(theta_1 + theta_2 + theta_3).sin() * theta_0.cos(),
            -theta_0.sin() * theta_4.sin()
                + theta_0.cos() * theta_4.cos() * (theta_1 + theta_2 + theta_3).cos(),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use nalgebra::Vector3;

    use crate::kinematics::forward::algorithms::analytical::AnalyticalForwardKinematicAlgorithm;
    use crate::kinematics::forward::algorithms::ForwardKinematicAlgorithm;
    use crate::kinematics::model::{KinematicParameters, KinematicState};

    #[test]
    pub fn solver_for_straight_pose() {
        // Create the kinematic state (in a way that all links point straight up).
        let state: KinematicState = KinematicState {
            theta_0: 0_f64,
            theta_1: 0_f64,
            theta_2: 0_f64,
            theta_3: 0_f64,
            theta_4: 0_f64,
        };

        // Create the default kinematic parameters.
        let params: KinematicParameters = KinematicParameters::default();

        // Create the analytical solver.
        let solver: AnalyticalForwardKinematicAlgorithm =
            AnalyticalForwardKinematicAlgorithm::default();

        // Make sure that the fourth limb position vector is correct.
        assert_eq!(
            solver.limb4_position_vector(&params, &state),
            Vector3::new(0_f64, params.sum_of_link_lengths(), 0_f64)
        );
    }
}
