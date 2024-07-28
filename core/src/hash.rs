use core::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Hash(
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    pub [u8; 32],
);

impl Hash {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.0 == [0u8; 32]
    }

    #[cfg(feature = "guest")]
    pub fn digest(value: &[u8]) -> Result<Self> {
        use risc0_zkvm::sha::{Impl, Sha256};

        Impl::hash_bytes(&value).as_bytes().try_into()
    }

    #[cfg(feature = "guest")]
    pub fn combine(a: Self, b: Self) -> Result<Self> {
        let mut value = [0u8; 64];

        value[..32].copy_from_slice(&a.0);
        value[32..].copy_from_slice(&b.0);

        Self::digest(&value)
    }

    #[cfg(feature = "guest")]
    pub fn combine3(a: Self, b: Self, c: Self) -> Result<Self> {
        let mut value = [0u8; 96];

        value[..32].copy_from_slice(&a.0);
        value[32..64].copy_from_slice(&b.0);
        value[64..].copy_from_slice(&c.0);

        Self::digest(&value)
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
