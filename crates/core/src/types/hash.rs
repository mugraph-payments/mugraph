use core::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result, SerializeBytes};

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Hash(
    #[cfg_attr(feature = "std", filter(#0 != [0u8; 32]))]
    #[serde(with = "hex::serde")]
    pub [u8; 32],
);

impl Hash {
    pub const SIZE: usize = 32;

    pub const fn new(input: [u8; Self::SIZE]) -> Self {
        Self(input)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::SIZE]
    }

    #[inline]
    #[cfg(not(feature = "std"))]
    pub fn digest<T: SerializeBytes>(hasher: &mut Sha256, value: &T) -> Result<Self> {
        let mut buf = [0u8; 512];
        let slice = &mut buf[0..T::SIZE];
        value.to_slice(slice);

        hasher.update(slice);

        let result = hasher.finalize_reset();
        result.as_slice().try_into()
    }

    #[inline]
    #[cfg(feature = "std")]
    pub fn digest<T: SerializeBytes>(hasher: &mut Sha256, value: &T) -> Result<Self> {
        let mut slice = Vec::with_capacity(T::SIZE);
        value.to_slice(&mut slice);

        hasher.update(slice);

        let result = hasher.finalize_reset();
        result.as_slice().try_into()
    }

    #[inline]
    pub fn combine(hasher: &mut Sha256, a: Self, b: Self) -> Result<Self> {
        hasher.update(a.0);
        hasher.update(b.0);
        let result = hasher.finalize_reset();
        result.as_slice().try_into()
    }

    #[inline]
    pub fn combine3(hasher: &mut Sha256, a: Self, b: Self, c: Self) -> Result<Self> {
        hasher.update(a.0);
        hasher.update(b.0);
        hasher.update(c.0);
        let result = hasher.finalize_reset();
        result.as_slice().try_into()
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hash {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 32]> for Hash {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<[u32; 8]> for Hash {
    #[inline]
    fn from(input: [u32; 8]) -> Self {
        let mut output = [0u8; 32];

        for (i, &value) in input.iter().enumerate() {
            output[i * 4] = (value & 0xFF) as u8;
            output[i * 4 + 1] = ((value >> 8) & 0xFF) as u8;
            output[i * 4 + 2] = ((value >> 16) & 0xFF) as u8;
            output[i * 4 + 3] = ((value >> 24) & 0xFF) as u8;
        }

        Hash(output)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = Error;

    #[inline]
    fn try_from(value: &[u8]) -> core::result::Result<Self, Self::Error> {
        assert_eq!(value.len(), 32);

        Ok(Self(value.try_into()?))
    }
}

impl SerializeBytes for Hash {
    const SIZE: usize = 32;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        out.copy_from_slice(&self.0)
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        assert!(input.len() >= 32);
        input.try_into()
    }
}

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut output = [0u8; 64];

        hex::encode_to_slice(self.0, &mut output).unwrap();
        core::str::from_utf8(&output).unwrap().fmt(f)
    }
}

impl core::fmt::Display for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

#[cfg(all(feature = "std", test))]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::Hash;
    use crate::SerializeBytes;

    #[proptest]
    fn test_serialize_bytes(input: Hash) {
        let mut buf = [0u8; 32];

        input.to_slice(&mut buf);
        let output = Hash::from_slice(&buf).unwrap();

        prop_assert_eq!(input, output);
    }

    #[proptest]
    fn test_try_from(input: [u8; 32]) {
        let input_ref: &[u8] = &input;
        let result: Hash = input_ref.try_into()?;

        prop_assert_eq!(result, Hash(input));
    }
}
