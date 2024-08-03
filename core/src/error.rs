use std::fmt::Display;

use onlyerror::Error;

use crate::types::Hash;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed note script validation for program {program_id}: {error}")]
    FailedScriptValidation { program_id: Hash, error: String },
    #[error("Failed CBOR encoding: {0}")]
    FailedCBOREncoding(String),
}

impl<T: Display> From<minicbor::encode::Error<T>> for Error {
    fn from(err: minicbor::encode::Error<T>) -> Self {
        Error::FailedCBOREncoding(err.to_string())
    }
}
