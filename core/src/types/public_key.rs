use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{traits::Public, Point, Scalar},
    error::Error,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PublicKey(#[serde(with = "serde_bytes")] pub [u8; 32]);

impl Arbitrary for PublicKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<[u8; 32]>()
            .prop_filter("must not be empty", |x| *x != [0u8; 32])
            .prop_map(Self)
            .boxed()
    }
}

impl Public for PublicKey {
    fn to_point(&self) -> Result<Point, Error> {
        self.to_compressed_point()?
            .decompress()
            .ok_or(Error::InvalidKey {
                reason: "Failed to decompress ristretto point".to_string(),
            })
    }
}

impl PublicKey {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn to_compressed_point(&self) -> Result<CompressedRistretto, Error> {
        CompressedRistretto::from_slice(&self.0).map_err(|e| Error::InvalidKey {
            reason: e.to_string(),
        })
    }

    #[inline]
    pub fn to_point(&self) -> Result<RistrettoPoint, Error> {
        self.to_compressed_point()?
            .decompress()
            .ok_or(Error::InvalidKey {
                reason: "Failed to decompress ristretto point".to_string(),
            })
    }

    #[inline]
    pub fn to_scalar(&self) -> Scalar {
        Scalar::from_bytes_mod_order(self.0)
    }
}

impl AsRef<[u8; 32]> for PublicKey {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Deref for PublicKey {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DerefMut for PublicKey {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl From<[u8; 32]> for PublicKey {
    #[inline]
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<[u32; 8]> for PublicKey {
    #[inline]
    fn from(data: [u32; 8]) -> Self {
        PublicKey(bytemuck::cast(data))
    }
}

impl From<RistrettoPoint> for PublicKey {
    #[inline]
    fn from(value: RistrettoPoint) -> Self {
        Self(value.compress().to_bytes())
    }
}

impl From<CompressedRistretto> for PublicKey {
    #[inline]
    fn from(value: CompressedRistretto) -> Self {
        Self(value.to_bytes())
    }
}

impl TryFrom<PublicKey> for CompressedRistretto {
    type Error = Error;

    fn try_from(value: PublicKey) -> Result<Self, Self::Error> {
        value.to_compressed_point()
    }
}

impl TryFrom<PublicKey> for RistrettoPoint {
    type Error = Error;

    #[inline]
    fn try_from(value: PublicKey) -> Result<Self, Self::Error> {
        value.to_point()
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut result = PublicKey::default();
        result.0.copy_from_slice(&value[..]);

        Ok(result)
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut result = PublicKey::default();
        result.0.copy_from_slice(value);

        Ok(result)
    }
}

impl LowerHex for PublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode(self.0), f)
    }
}

impl UpperHex for PublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.0), f)
    }
}

impl core::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}
