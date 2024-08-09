use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidUnblindedPoint,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("ZKVM error")]
    ZKVM,

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] risc0_zkvm::serde::Error),
}
