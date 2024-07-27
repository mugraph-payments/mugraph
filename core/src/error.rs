use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to write value to guest")]
    FailedToWriteValue,

    #[error("Failed to initialize executor")]
    FailedToInitializeExecutor,
}
