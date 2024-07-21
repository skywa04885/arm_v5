use nalgebra::Vector3;

use super::Motion;

/// This struct represents a linear motion from the original position to the target position.
pub(crate) struct LinearMotion {
    target_position: Vector3<f64>,   // The target position (in meters).
    original_position: Vector3<f64>, // The original position (in meters).
    speed: f64,                      // The speed (in meters/second).
}

impl Motion for LinearMotion {
    /// Interpolates the position at a given time.
    ///
    /// # Arguments
    ///
    /// * `t` - The time value (in seconds).
    ///
    /// # Returns
    ///
    /// * `Some(Vector3<f64>)` - The interpolated position if `t` is within the motion duration.
    /// * `None` - If `t` is greater than the motion duration.
    fn interpolate(&self, t: f64) -> Option<nalgebra::Vector3<f64>> {
        assert!(t >= 0_f64);

        // Calculate the change in position from the original position to the target position.
        let delta_position = self.original_position - self.target_position;

        // Calculate the duration of the motion based on the magnitude of the delta position and the speed.
        let duration = delta_position.magnitude() / self.speed;

        // If the given time is greater than the duration of the motion, return None.
        if t > duration {
            return None;
        }

        // Calculate the delta change in position per unit of time.
        let delta = delta_position / duration;

        // Calculate the interpolated position at the given time.
        Some(delta * t)
    }
}
