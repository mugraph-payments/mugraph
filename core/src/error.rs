use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to write value to executor")]
    ExecutorWriteValue,

    #[error("Failed to initialize executor")]
    ExecutorInitialize,

    #[error("Failed to generate proof")]
    ProofGenerate,

    #[error("Failed to decode journal")]
    JournalDecode,

    #[error("Failed to decode standard output")]
    StdoutDecode,

    #[error("Invalid Hash")]
    InvalidHash,
}
