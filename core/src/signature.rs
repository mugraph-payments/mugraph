use crate::{Error, Point, Scalar};
use curve25519_dalek::ristretto::CompressedRistretto;
use hex::{serde::deserialize as hex_deserialize, serde::serialize as hex_serialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
    pub r: Point,
    pub s: Scalar,
}

impl Signature {
    fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&self.r.compress().to_bytes());
        bytes[32..64].copy_from_slice(&self.s.to_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 64 {
            return Err(Error::InvalidSignature);
        }

        let r = CompressedRistretto::from_slice(&bytes[0..32])
            .map_err(|_| Error::InvalidSignature)?
            .decompress()
            .ok_or(Error::InvalidSignature)?;

        let s = Scalar::from_slice(&bytes[32..64]).map_err(|_| Error::InvalidSignature)?;

        Ok(Signature { r, s })
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
        Signature::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
