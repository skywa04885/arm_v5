use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Communication error: {0}")]
    ComError(#[from] com::error::Error),
}
