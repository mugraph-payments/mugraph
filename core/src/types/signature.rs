use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use curve25519_dalek::ristretto::CompressedRistretto;
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{
    crypto::Point,
    error::{Error, Result},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Arbitrary)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Blinded<T>(pub T);

#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    Arbitrary,
    PartialOrd,
    Ord,
)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Signature(pub [u8; 32]);

impl Signature {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn to_point(self) -> Result<Point> {
        CompressedRistretto::from_slice(&self.0)
            .map_err(|e| Error::InvalidSignature {
                reason: e.to_string(),
                signature: self,
            })?
            .decompress()
            .ok_or(Error::InvalidSignature {
                reason: "failed to decompress ristretto point".to_string(),
                signature: self,
            })
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
        Display::fmt(&muhex::encode(self.0), f)
    }
}

impl UpperHex for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.0).to_uppercase(), f)
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

impl From<Point> for Signature {
    #[inline]
    fn from(value: Point) -> Self {
        Self(value.compress().to_bytes())
    }
}

impl redb::Key for Signature {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl redb::Value for Signature {
    type SelfType<'a>
        = Self
    where
        Self: 'a;
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(32)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(data);
        Self(arr)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        &value.0
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("signature")
    }
}
