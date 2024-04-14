use nalgebra::{Vector2, Vector3};

use super::Motion;

/// Represents a circular motion.
/// 
/// The yaw rotation is around the $y$ axis, and the pitch rotation around the $x$ axis.
pub struct CircleMotion {
    center_position: Vector3<f64>, // Position of the center of the circle in meters
    orientation: Vector2<f64>, // Orientation vector representing pitch and yaw in radians
    radius: f64, // Radius of the circle in meters
    angular_velocity: f64, // Angular velocity of the circle in meters/second
    laps: f64, // The number of laps around the circle.
}
impl Motion for CircleMotion {
    fn interpolate(&self, t: f64) -> Option<nalgebra::Vector3<f64>> {
        None
    }
}