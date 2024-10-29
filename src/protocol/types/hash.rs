use std::fmt;

use curve25519_dalek::{constants, ristretto::CompressedRistretto, RistrettoPoint, Scalar};
use plonky2::{hash::hash_types::HashOut, plonk::config::GenericHashOut};
use rand::{prelude::*, rngs::OsRng};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{protocol::circuit::*, Decode, DecodeFields, Encode, EncodeFields, Error};

#[derive(
    Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Arbitrary, Hash,
)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Hash(#[serde(with = "hex::serde")] [u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn new(val: [u8; 32]) -> Self {
        Self(val)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid slice length for Hash: expected 32, got {}",
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

    pub fn random() -> Self {
        Self(OsRng.gen())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<[u64; 4]> for Hash {
    fn from(value: [u64; 4]) -> Self {
        let mut bytes = [0u8; 32];

        for (i, &val) in value.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&val.to_le_bytes());
        }

        Self(bytes)
    }
}

impl From<&[u64]> for Hash {
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

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Hash> for Scalar {
    type Error = Error;

    fn try_from(hash: Hash) -> Result<Self, Error> {
        if hash.as_bytes()[31] != 0 {
            Err(Error::DecodeError("Not a valid scalar, last byte should be zero to ensure it fits inside the group modulo.".to_string()))
        } else {
            Ok(Scalar::from_bytes_mod_order(hash.0))
        }
    }
}

impl From<Scalar> for Hash {
    fn from(scalar: Scalar) -> Self {
        Self(scalar.to_bytes())
    }
}

impl From<RistrettoPoint> for Hash {
    fn from(point: RistrettoPoint) -> Self {
        Self(point.compress().to_bytes())
    }
}

impl TryFrom<Hash> for RistrettoPoint {
    type Error = Error;

    fn try_from(hash: Hash) -> Result<Self, Self::Error> {
        CompressedRistretto::from_slice(&hash.0)
            .map_err(|e| Error::DecodeError(e.to_string()))?
            .decompress()
            .ok_or(Error::DecodeError("Could not decompress point".to_string()))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<HashOut<F>> for Hash {
    fn from(value: HashOut<F>) -> Self {
        Self::from_bytes(&value.to_bytes()).unwrap()
    }
}

impl From<Hash> for HashOut<F> {
    fn from(value: Hash) -> Self {
        HashOut {
            elements: value.as_fields().try_into().unwrap(),
        }
    }
}

impl Encode for Hash {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl EncodeFields for Hash {
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl Decode for Hash {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid length for Hash: expected 32, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl DecodeFields for Hash {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 4 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for Hash: expected 4, got {}",
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
    use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT as G, RistrettoPoint, Scalar};
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::Hash;
    use crate::{test_encode_bytes, test_encode_fields, Encode};

    test_encode_bytes!(Hash);
    test_encode_fields!(Hash);

    fn scalar() -> impl Strategy<Value = Hash> {
        any::<Hash>()
            .prop_map(|x| {
                let mut bytes = x.as_bytes();
                bytes[31] = 0;

                bytes
            })
            .prop_map(|x| Hash::from_slice(&x).unwrap())
    }

    fn point() -> impl Strategy<Value = Hash> {
        scalar()
            .prop_map(|x| Scalar::try_from(x).unwrap() * G)
            .prop_map(Hash::from)
    }

    #[proptest]
    fn test_hash_scalar_roundtrip(#[strategy(scalar())] hash: Hash) {
        prop_assert_eq!(Hash::from(Scalar::try_from(hash)?), hash);
    }

    #[proptest]
    fn test_hash_point_roundtrip(#[strategy(point())] hash: Hash) {
        prop_assert_eq!(Hash::from(RistrettoPoint::try_from(hash)?), hash);
    }
}
