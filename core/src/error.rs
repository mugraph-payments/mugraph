use std::fmt::Display;

use onlyerror::Error;

use crate::types::Hash;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum VMError {}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed note script validation for program {program_id}: {error}")]
    ScriptValidation { program_id: Hash, error: String },

    #[error("Failed CBOR encoding: {0}")]
    CBOR(String),

    #[error("Serializaition/Deserialization error: {0}")]
    Serde(#[from] risc0_zkvm::serde::Error),

    #[error("ZKVM error: {0}")]
    ZKVM(String),
}

impl<T: Display> From<minicbor::encode::Error<T>> for Error {
    fn from(err: minicbor::encode::Error<T>) -> Self {
        Error::CBOR(err.to_string())
    }
}
