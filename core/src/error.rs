#[cfg(feature = "std")]
use std::array::TryFromSliceError;

#[cfg(not(feature = "std"))]
use core::array::TryFromSliceError;

use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Hash")]
    InvalidHash,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Failed Deserialization")]
    FailedDeserialization,

    #[error("Invalid unblinded point")]
    InvalidUnblindedPoint,
}

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Self {
        Self::FailedDeserialization
    }
}
