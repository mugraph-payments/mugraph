use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use curve25519_dalek::ristretto::CompressedRistretto;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::Point,
    error::{Error, Result},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[repr(transparent)]
#[serde(transparent)]
pub struct Signature(#[serde(with = "serde_bytes")] pub [u8; 32]);

impl Signature {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn to_point(self) -> Result<Point> {
        CompressedRistretto::from_slice(&self.0)
            .map_err(|_| Error::InvalidPoint)?
            .decompress()
            .ok_or(Error::InvalidPoint)
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self([0u8; 32])
    }
}

impl core::fmt::Display for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl LowerHex for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode(self.0), f)
    }
}

impl UpperHex for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.0), f)
    }
}

impl AsRef<[u8; 32]> for Signature {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Deref for Signature {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DerefMut for Signature {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl From<[u8; 32]> for Signature {
    #[inline]
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}
