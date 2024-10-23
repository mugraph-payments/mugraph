use std::fmt;

use curve25519_dalek::{edwards::CompressedEdwardsY, EdwardsPoint};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{protocol::*, DecodeFields, EncodeFields, Error};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PublicKey(#[serde(with = "hex::serde")] [u8; 32]);

impl PublicKey {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn from_point(point: EdwardsPoint) -> Self {
        Self(point.compress().to_bytes())
    }
}

impl Encode for PublicKey {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl EncodeFields for PublicKey {
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl Decode for PublicKey {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid length for PublicKey: expected 32, got {}",
                bytes.len()
            )));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl DecodeFields for PublicKey {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 4 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for PublicKey: expected 4, got {}",
                fields.len()
            )));
        }
        let mut bytes = [0u8; 32];
        for (i, field) in fields.iter().enumerate() {
            let field_bytes = field.to_canonical_u64().to_le_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&field_bytes);
        }
        Ok(Self(bytes))
    }
}

impl From<[u8; 32]> for PublicKey {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for PublicKey {
    fn from(slice: &[u8]) -> Self {
        if slice.len() != 32 {
            panic!(
                "Invalid slice length for PublicKey: expected 32, got {}",
                slice.len()
            );
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);

        Self(hash)
    }
}

impl From<EdwardsPoint> for PublicKey {
    fn from(point: EdwardsPoint) -> Self {
        Self(point.compress().to_bytes())
    }
}

impl From<PublicKey> for EdwardsPoint {
    fn from(public_key: PublicKey) -> Self {
        CompressedEdwardsY(public_key.0)
            .decompress()
            .expect("Invalid public key")
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Arbitrary for PublicKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        Hash::arbitrary_with(params)
            .prop_map(|x| Self(x.as_ref().try_into().unwrap()))
            .boxed()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::PublicKey;

    crate::test_encode_bytes!(PublicKey);
    crate::test_encode_fields!(PublicKey);
}