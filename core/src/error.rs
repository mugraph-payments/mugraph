use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed CBOR encoding")]
    CBOR,

    #[error("ZKVM error")]
    ZKVM,

    #[error("Serializaition/Deserialization error: {0}")]
    Serde(#[from] risc0_zkvm::serde::Error),
}

impl<T: core::fmt::Display> From<minicbor::encode::Error<T>> for Error {
    fn from(_: minicbor::encode::Error<T>) -> Self {
        Error::CBOR
    }
}
