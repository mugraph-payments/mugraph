use std::{fmt, str};

use plonky2::hash::hash_types::HashOut;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use super::Hash;
use crate::{protocol::circuit::*, Decode, DecodeFields, Encode, EncodeFields, Error};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Name(#[serde(with = "hex::serde")] [u8; 32]);

impl Name {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
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
}

impl From<[u8; 32]> for Name {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Name {
    fn from(slice: &[u8]) -> Self {
        if slice.len() != 32 {
            panic!(
                "Invalid slice length for Name: expected 32, got {}",
                slice.len()
            );
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);

        Self(hash)
    }
}

impl AsRef<[u8]> for Name {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Arbitrary for Name {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        Hash::arbitrary_with(params)
            .prop_map(|x| Self(x.as_ref().try_into().unwrap()))
            .boxed()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{}", s.trim_end_matches('\0')),
            Err(_) => {
                for byte in self.0.iter() {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl From<HashOut<F>> for Name {
    fn from(value: HashOut<F>) -> Self {
        Self(Hash::from(value).as_ref().try_into().unwrap())
    }
}

impl From<Name> for HashOut<F> {
    fn from(value: Name) -> Self {
        Hash::new(value.as_ref().try_into().unwrap()).into()
    }
}

impl Encode for Name {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for Name {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid length for Name: expected 32, got {}",
                bytes.len()
            )));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl EncodeFields for Name {
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl DecodeFields for Name {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 4 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for Name: expected 4, got {}",
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
    use super::Name;

    crate::test_encode_bytes!(Name);
    crate::test_encode_fields!(Name);
}
