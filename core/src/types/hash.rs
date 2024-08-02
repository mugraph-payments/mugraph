use std::ops::{Deref, DerefMut};

use minicbor::{Decode, Encode};
use risc0_zkvm::sha::Digest;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[serde(transparent)]
#[cbor(transparent)]
pub struct Hash(#[n(0)] [u8; 32]);

impl Hash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.as_ref()
    }
}

impl AsRef<[u8; 32]> for Hash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Deref for Hash {
    type Target = [u8; 32];
    fn deref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl DerefMut for Hash {
    fn deref_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<risc0_zkvm::sha::Digest> for Hash {
    fn from(value: Digest) -> Self {
        let bytes = value.as_bytes();
        assert_eq!(bytes.len(), 32);

        let mut result = Hash::default();
        result.0.copy_from_slice(bytes);
        result
    }
}

impl From<Hash> for risc0_zkvm::sha::Digest {
    fn from(value: Hash) -> Self {
        Self::from(value.0)
    }
}

impl From<[u32; 8]> for Hash {
    fn from(data: [u32; 8]) -> Self {
        Hash(*bytemuck::cast_ref(&data))
    }
}
