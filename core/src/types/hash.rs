use core::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result, SerializeBytes};

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Hash(#[serde(with = "hex::serde")] pub [u8; 32]);

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
    pub fn digest(value: &[u8]) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize();
        result.as_slice().try_into()
    }

    #[inline]
    pub fn combine(a: Self, b: Self) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(a.0);
        hasher.update(b.0);
        let result = hasher.finalize();
        result.as_slice().try_into()
    }

    #[inline]
    pub fn combine3(a: Self, b: Self, c: Self) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(a.0);
        hasher.update(b.0);
        hasher.update(c.0);
        let result = hasher.finalize();
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

impl TryFrom<&[u8]> for Hash {
    type Error = Error;

    #[inline]
    fn try_from(value: &[u8]) -> core::result::Result<Self, Self::Error> {
        debug_assert_eq!(value.len(), 32);

        if value.len() != 32 {
            return Err(Error::FailedDeserialization);
        }

        let bytes: [u8; 32] = unsafe { *(value.as_ptr() as *const [u8; 32]) };

        Ok(Self(bytes))
    }
}

impl SerializeBytes for Hash {
    const SIZE: usize = <[u8; 32]>::SIZE;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        debug_assert!(out.len() >= 32);

        out.copy_from_slice(&self.0)
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
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

#[cfg(all(feature = "std", test))]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::Hash;

    #[proptest]
    fn test_try_from(input: [u8; 32]) {
        let input_ref: &[u8] = &input;
        let result: Hash = input_ref.try_into()?;

        prop_assert_eq!(result, Hash(input));
    }

    #[proptest]
    fn test_try_from(input: [u8; 32]) {
        let input_ref: &[u8] = &input;
        let result: Hash = input_ref.try_into()?;

        prop_assert_eq!(result, Hash(input));
    }
}
