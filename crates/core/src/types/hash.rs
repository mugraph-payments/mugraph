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

impl From<[u32; 8]> for Hash {
    fn from(value: [u32; 8]) -> Self {
        let mut result = [0u8; 32];

        for (i, &num) in value.iter().enumerate() {
            let bytes = num.to_le_bytes();
            result[i * 4] = bytes[0];
            result[i * 4 + 1] = bytes[1];
            result[i * 4 + 2] = bytes[2];
            result[i * 4 + 3] = bytes[3];
        }

        result.into()
    }
}
