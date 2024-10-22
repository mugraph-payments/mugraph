use std::fmt;

use plonky2::{hash::hash_types::HashOut, plonk::config::GenericHashOut};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{protocol::*, Decode, DecodeFields, Encode, EncodeFields, Error};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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

    #[allow(clippy::self_named_constructors)]
    pub fn hash(input: &[u8]) -> Self {
        Self(blake3::hash(input).into())
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

impl Arbitrary for Hash {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<Vec<u8>>().prop_map(|x| Hash::hash(&x)).boxed()
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
        Self::from_slice(&value.to_bytes()).unwrap()
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
    use super::Hash;
    use crate::{test_encode_bytes, test_encode_fields};

    test_encode_bytes!(Hash);
    test_encode_fields!(Hash);
}
