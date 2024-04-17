use std::borrow::Cow;

use thiserror::Error;

use crate::kinematics::error::KinematicError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Communication error: {0}")]
    ComError(#[from] com::error::Error),
    #[error("{0}")]
    Generic(Cow<'static, str>),
    #[error("Kinematic error: {0}")]
    KinematicError(#[from] KinematicError)
}
