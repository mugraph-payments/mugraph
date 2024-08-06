use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use risc0_zkvm::sha::{Digest, Impl, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Hash(#[serde(with = "serde_bytes")] [u8; 32]);

#[cfg(feature = "proptest")]
impl proptest::arbitrary::Arbitrary for Hash {
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

impl Hash {
    #[inline]
    pub fn zero() -> Self {
        Self::default()
    }

    #[inline]
    pub fn digest(input: &[u8]) -> Self {
        (*Impl::hash_bytes(input)).into()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.as_ref()
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

impl From<risc0_zkvm::sha::Digest> for Hash {
    #[inline]
    fn from(value: Digest) -> Self {
        let bytes = value.as_bytes();
        assert_eq!(bytes.len(), 32);

        let mut result = Hash::default();
        result.0.copy_from_slice(bytes);
        result
    }
}

impl From<Hash> for risc0_zkvm::sha::Digest {
    #[inline]
    fn from(value: Hash) -> Self {
        Self::from(value.0)
    }
}

impl From<[u32; 8]> for Hash {
    #[inline]
    fn from(data: [u32; 8]) -> Self {
        Hash(*bytemuck::cast_ref(&data))
    }
}

#[cfg(feature = "std")]
impl TryFrom<Vec<u8>> for Hash {
    type Error = crate::error::Error;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        assert_eq!(value.len(), 32);

        let mut result = Hash::default();
        result.0.copy_from_slice(&value[..]);

        Ok(result)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = crate::error::Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        assert_eq!(value.len(), 32);

        let mut result = Hash::default();
        result.0.copy_from_slice(value);

        Ok(result)
    }
}

impl LowerHex for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode(self.0), f)
    }
}

impl UpperHex for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.0), f)
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
