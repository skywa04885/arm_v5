use nalgebra::Vector3;

pub(crate) mod linear;
pub(crate) mod circle;
pub(crate) mod player;

pub(crate) trait Motion: Send {
    /// Interpolate the motion at the given timestamp, return the new end-effector position
    ///  or None if the motion is finished.
    fn interpolate(&self, t: f64) -> Option<Vector3<f64>>;
}
