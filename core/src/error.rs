use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid unblinded point")]
    InvalidPoint,

    #[error("Invalid public or secret key")]
    InvalidKey,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("ZKVM error")]
    ZKVM,

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] risc0_zkvm::serde::Error),
}
