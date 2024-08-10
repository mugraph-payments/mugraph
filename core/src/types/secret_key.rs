use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use curve25519_dalek::Scalar;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SecretKey(#[serde(with = "serde_bytes")] pub [u8; 32]);

#[cfg(feature = "proptest")]
impl proptest::arbitrary::Arbitrary for SecretKey {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;

        proptest::collection::vec(any::<u8>(), 32)
            .prop_filter("must not be empty", |x| x.as_slice() != &[0u8; 32])
            .prop_map(|x| Self::try_from(x.as_ref()).unwrap())
            .boxed()
    }
}

impl SecretKey {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn to_scalar(&self) -> Result<Scalar, Error> {
        let result: Option<_> = Scalar::from_canonical_bytes(self.0).into();
        result.ok_or(Error::InvalidKey)
    }
}

impl AsRef<[u8; 32]> for SecretKey {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Deref for SecretKey {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DerefMut for SecretKey {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl From<[u8; 32]> for SecretKey {
    #[inline]
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<[u32; 8]> for SecretKey {
    #[inline]
    fn from(data: [u32; 8]) -> Self {
        SecretKey(bytemuck::cast(data))
    }
}

impl From<Scalar> for SecretKey {
    #[inline]
    fn from(value: Scalar) -> Self {
        SecretKey(value.to_bytes())
    }
}

impl TryFrom<SecretKey> for Scalar {
    type Error = Error;

    fn try_from(value: SecretKey) -> Result<Self, Self::Error> {
        value.to_scalar()
    }
}

#[cfg(feature = "std")]
impl TryFrom<Vec<u8>> for SecretKey {
    type Error = Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut result = SecretKey::default();
        result.0.copy_from_slice(&value[..]);

        Ok(result)
    }
}

impl TryFrom<&[u8]> for SecretKey {
    type Error = Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut result = SecretKey::default();
        result.0.copy_from_slice(value);

        Ok(result)
    }
}

impl LowerHex for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode(self.0), f)
    }
}

impl UpperHex for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.0), f)
    }
}

impl core::fmt::Display for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}
