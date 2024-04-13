use nalgebra::Vector5;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KinematicParameters {
    pub l_0: f64,
    pub l_1: f64,
    pub l_2: f64,
    pub l_3: f64,
    pub l_4: f64,
}

impl KinematicParameters {
    /// Compute the sum of all the link lengths.
    pub fn sum_of_link_lengths(&self) -> f64 {
        self.l_0 + self.l_1 + self.l_2 + self.l_3 + self.l_4
    }
}

impl Default for KinematicParameters {
    fn default() -> Self {
        Self {
            l_0: 10_f64,
            l_1: 10_f64,
            l_2: 10_f64,
            l_3: 10_f64,
            l_4: 10_f64,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KinematicState {
    pub theta_0: f64,
    pub theta_1: f64,
    pub theta_2: f64,
    pub theta_3: f64,
    pub theta_4: f64,
}

impl Default for KinematicState {
    fn default() -> Self {
        Self {
            theta_0: 0.2_f64,
            theta_1: 0.2_f64,
            theta_2: 0.2_f64,
            theta_3: 0.2_f64,
            theta_4: 0.2_f64,
        }
    }
}

impl From<Vector5<f64>> for KinematicState {
    fn from(value: Vector5<f64>) -> Self {
        Self {
            theta_0: value.x,
            theta_1: value.y,
            theta_2: value.z,
            theta_3: value.w,
            theta_4: value.a,
        }
    }
}

impl From<&KinematicState> for Vector5<f64> {
    fn from(value: &KinematicState) -> Self {
        Vector5::<f64>::new(
            value.theta_0,
            value.theta_1,
            value.theta_2,
            value.theta_3,
            value.theta_4,
        )
    }
}
