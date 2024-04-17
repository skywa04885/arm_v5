use thiserror::Error;

#[derive(Error, Debug)]
pub enum KinematicError {
    #[error("Inversion failure")]
    InversionFailure,
}