use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("Failed to write value to guest: {0}")]
    FailedToWriteValue(String),

    #[error("Failed to initialize executor: {0}")]
    FailedToInitializeExecutor(String),
}
