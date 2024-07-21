use std::borrow::Cow;

use kinematics::error::KinematicError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Communication error: {0}")]
    ComError(#[from] com::error::Error),
    #[error("{0}")]
    Generic(Cow<'static, str>),
    #[error("Kinematic error: {0}")]
    KinematicError(#[from] KinematicError)
}
