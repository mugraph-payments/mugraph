use std::{
    array::TryFromSliceError,
    num::{ParseIntError, TryFromIntError},
};

use thiserror::Error as ThisError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
pub enum CRDTError {
    #[error("invalid operation")]
    InvalidOperation,

    #[error("invalid state")]
    InvalidState,
}

#[derive(Debug, ThisError, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("failed to deserialize: {0}")]
    FailedDeserialization(String),

    #[error("key is already present")]
    AlreadyPresent,

    #[error("CRDT error: {0}")]
    CRDT(#[from] CRDTError),

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl From<hex::FromHexError> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(error: hex::FromHexError) -> Self {
        Error::FailedDeserialization(format!("{}", error))
    }
}

impl From<ParseIntError> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(error: ParseIntError) -> Self {
        Error::FailedDeserialization(format!("{}", error))
    }
}

impl From<TryFromIntError> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(error: TryFromIntError) -> Self {
        Error::FailedDeserialization(format!("invalid number format: {}", error))
    }
}

impl From<TryFromSliceError> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(error: TryFromSliceError) -> Self {
        Error::FailedDeserialization(format!("invalid slice format: {}", error))
    }
}

impl From<ed25519_dalek::SignatureError> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(error: ed25519_dalek::SignatureError) -> Self {
        Error::FailedDeserialization(format!("invalid signature format: {}", error))
    }
}
