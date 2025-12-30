use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use blake3::{Hash as Blake3Hash, Hasher};
use curve25519_dalek::ristretto::CompressedRistretto;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use crate::crypto::Scalar;

#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Hash(#[serde(with = "muhex::serde")] pub [u8; 32]);

impl Arbitrary for Hash {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<[u8; 32]>()
            .prop_filter("must not be empty", |x| *x != [0u8; 32])
            .prop_map(Self)
            .boxed()
    }
}

impl Hash {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    #[inline]
    pub fn digest(input: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(input);
        let result = hasher.finalize();

        Self(result.into())
    }

    pub fn random<R: RngCore>(rng: &mut R) -> Self {
        let mut output = [0u8; 32];
        rng.fill_bytes(&mut output);

        Self(output)
    }

    #[inline]
    pub fn to_scalar(&self) -> Scalar {
        Scalar::from_bytes_mod_order(self.0)
    }
}

impl AsRef<[u8; 32]> for Hash {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DerefMut for Hash {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl From<[u8; 32]> for Hash {
    #[inline]
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<[u32; 8]> for Hash {
    #[inline]
    fn from(data: [u32; 8]) -> Self {
        Hash(bytemuck::cast(data))
    }
}

impl From<Blake3Hash> for Hash {
    fn from(value: Blake3Hash) -> Self {
        Hash(*value.as_bytes())
    }
}

impl From<Scalar> for Hash {
    fn from(value: Scalar) -> Self {
        value.to_bytes().into()
    }
}

impl From<Hash> for Scalar {
    fn from(value: Hash) -> Self {
        Scalar::from_bytes_mod_order(value.0)
    }
}

impl From<CompressedRistretto> for Hash {
    fn from(value: CompressedRistretto) -> Self {
        Self(*value.as_bytes())
    }
}

impl From<Hash> for CompressedRistretto {
    fn from(value: Hash) -> Self {
        CompressedRistretto::from_slice(&value.0).unwrap()
    }
}

impl TryFrom<Vec<u8>> for Hash {
    type Error = crate::error::Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut result = Hash::default();
        result.0.copy_from_slice(&value[..]);

        Ok(result)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = crate::error::Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut result = Hash::default();
        result.0.copy_from_slice(value);

        Ok(result)
    }
}

impl LowerHex for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.0), f)
    }
}

impl UpperHex for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.0).to_uppercase(), f)
    }
}

impl core::fmt::Display for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::Value;
    use test_strategy::proptest;

    use super::Hash;

    #[proptest]
    fn test_serialization(value: Hash) {
        prop_assert!(matches!(
            serde_json::to_value(value).unwrap(),
            Value::String(_)
        ))
    }
}
