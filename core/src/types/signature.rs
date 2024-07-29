use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Error, Hash};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    pub r: Hash,
    pub s: Hash,
}

impl Signature {
    pub fn is_empty(&self) -> bool {
        self.r.is_empty() || self.s.is_empty()
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&*self.r);
        bytes[32..64].copy_from_slice(&*self.s);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
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
        serde_bytes::serialize(&self.to_bytes(), serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 64] = serde_bytes::deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
