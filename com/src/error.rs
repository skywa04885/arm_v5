use std::borrow::Cow;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    Generic(Cow<'static, str>),
    #[error("Operation cancelled")]
    Cancelled,
}
