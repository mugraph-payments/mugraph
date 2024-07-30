use core::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result, SerializeBytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Hash(#[serde(with = "hex::serde")] pub [u8; 32]);

impl Hash {
    pub const SIZE: usize = 32;

    pub const fn new(input: [u8; Self::SIZE]) -> Self {
        Self(input)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::SIZE]
    }

    pub fn digest(value: &[u8]) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(value);
        let result = hasher.finalize();
        result.as_slice().try_into()
    }

    #[inline(always)]
    pub fn combine(a: Self, b: Self) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(a.0);
        hasher.update(b.0);
        let result = hasher.finalize();
        result.as_slice().try_into()
    }

    #[inline(always)]
    pub fn combine3(a: Self, b: Self, c: Self) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(a.0);
        hasher.update(b.0);
        hasher.update(c.0);
        let result = hasher.finalize();
        result.as_slice().try_into()
    }

    #[inline(always)]
    pub fn combine4(a: Self, b: Self, c: Self, d: Self) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(a.0);
        hasher.update(b.0);
        hasher.update(c.0);
        hasher.update(d.0);
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

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = Error;

    fn try_from(value: &[u8]) -> core::result::Result<Self, Self::Error> {
        if value.len() != 32 {
            return Err(Error::InvalidHash);
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(value);

        Ok(Self(bytes))
    }
}

impl SerializeBytes for Hash {
    const SIZE: usize = <[u8; 32]>::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        out.copy_from_slice(&self.0)
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        input.try_into()
    }
}
