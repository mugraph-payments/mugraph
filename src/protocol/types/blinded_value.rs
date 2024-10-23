use std::fmt;

use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint, Scalar};
use plonky2::{hash::hash_types::HashOut, plonk::config::GenericHashOut};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{protocol::*, Decode, DecodeFields, Encode, EncodeFields, Error};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Arbitrary)]
#[repr(transparent)]
#[serde(transparent)]
pub struct BlindedValue(#[serde(with = "hex::serde")] [u8; 32]);

impl BlindedValue {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn new(val: [u8; 32]) -> Self {
        Self(val)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid slice length for BlindedValue: expected 32, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);

        Ok(Self(array))
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn inner(&self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for BlindedValue {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<[u64; 4]> for BlindedValue {
    fn from(value: [u64; 4]) -> Self {
        let mut bytes = [0u8; 32];

        for (i, &val) in value.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&val.to_le_bytes());
        }

        Self(bytes)
    }
}

impl From<&[u64]> for BlindedValue {
    fn from(slice: &[u64]) -> Self {
        let mut bytes = [0u8; 32];

        for (i, &val) in slice.iter().enumerate().take(4) {
            let start = i * 8;
            let end = (i + 1) * 8;

            if end <= bytes.len() {
                bytes[start..end].copy_from_slice(&val.to_le_bytes());
            } else {
                break;
            }
        }
        Self(bytes)
    }
}

impl AsRef<[u8]> for BlindedValue {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Scalar> for BlindedValue {
    fn from(scalar: Scalar) -> Self {
        Self(scalar.to_bytes())
    }
}

impl From<BlindedValue> for Scalar {
    fn from(hash: BlindedValue) -> Self {
        Scalar::from_bytes_mod_order(hash.0)
    }
}

impl From<RistrettoPoint> for BlindedValue {
    fn from(point: RistrettoPoint) -> Self {
        Self(point.compress().to_bytes())
    }
}

impl From<BlindedValue> for RistrettoPoint {
    fn from(hash: BlindedValue) -> Self {
        CompressedRistretto::from_slice(&hash.0)
            .unwrap()
            .decompress()
            .unwrap()
    }
}

impl fmt::Display for BlindedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for BlindedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<HashOut<F>> for BlindedValue {
    fn from(value: HashOut<F>) -> Self {
        Self::from_slice(&value.to_bytes()).unwrap()
    }
}

impl From<BlindedValue> for HashOut<F> {
    fn from(value: BlindedValue) -> Self {
        HashOut {
            elements: value.as_fields().try_into().unwrap(),
        }
    }
}

impl Encode for BlindedValue {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl EncodeFields for BlindedValue {
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl Decode for BlindedValue {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid length for BlindedValue: expected 32, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl DecodeFields for BlindedValue {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 4 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for BlindedValue: expected 4, got {}",
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

#[cfg(test)]
mod tests {
    use super::BlindedValue;
    use crate::{test_encode_bytes, test_encode_fields};

    test_encode_bytes!(BlindedValue);
    test_encode_fields!(BlindedValue);
}
