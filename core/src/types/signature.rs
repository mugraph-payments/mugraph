use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
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
        let mut w = Writer::new(out);

        w.write(&self.r);
        w.write(&self.s);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);

        Ok(Self {
            r: r.read()?,
            s: r.read()?,
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
