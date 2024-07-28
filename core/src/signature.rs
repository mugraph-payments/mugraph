use hex::{serde::deserialize as hex_deserialize, serde::serialize as hex_serialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Error, Hash};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
    pub r: Hash,
    pub s: Hash,
}

impl Signature {
    fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&*self.r);
        bytes[32..64].copy_from_slice(&*self.s);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 64 {
            return Err(Error::InvalidSignature);
        }

        let mut this = Self::default();
        this.r.0.copy_from_slice(&bytes[..32]);
        this.s.0.copy_from_slice(&bytes[32..]);

        Ok(this)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hex_serialize(&self.to_bytes(), serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 64] = hex_deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
