use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    pub r: Hash,
    pub s: Hash,
}

impl Signature {
    pub fn is_empty(&self) -> bool {
        self.r.is_empty() || self.s.is_empty()
    }
}

impl SerializeBytes for Signature {
    const SIZE: usize = Hash::SIZE * 2;

    fn to_slice(&self, out: &mut [u8]) {
        self.r.to_slice(&mut out[..Hash::SIZE]);
        self.s.to_slice(&mut out[Hash::SIZE..]);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        if input.len() < 64 {
            return Err(Error::InvalidSignature);
        }

        let mut this = Self::default();
        this.r.0.copy_from_slice(&input[..32]);
        this.s.0.copy_from_slice(&input[32..]);

        Ok(Self {
            r: Hash::from_slice(&input[..32])?,
            s: Hash::from_slice(&input[32..])?,
        })
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = [0u8; Self::SIZE];
        self.to_slice(&mut buf);
        hex::serde::serialize(&buf, serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 64] = hex::serde::deserialize(deserializer)?;
        Self::from_slice(&bytes).map_err(serde::de::Error::custom)
    }
}
