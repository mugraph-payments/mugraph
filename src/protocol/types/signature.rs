use plonky2::field::{goldilocks_field::GoldilocksField, types::Field};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use super::Hash;
use crate::{protocol::*, Decode, DecodeFields, Encode, Error};

// A Schnorr signature for a value
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Arbitrary, Deserialize, Serialize,
)]
pub struct Signature {
    pub r: Hash,
    pub s: Hash,
}

impl Signature {
    pub fn zero() -> Self {
        Self {
            r: Hash::zero(),
            s: Hash::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.r.is_zero() && self.s.is_zero()
    }
}

impl Encode for Signature {
    fn as_bytes(&self) -> Vec<u8> {
        [self.r.as_bytes(), self.s.as_bytes()].concat()
    }
}

impl EncodeFields for Signature {
    fn as_fields(&self) -> Vec<F> {
        let r = self.r.as_fields();
        let s = self.s.as_fields();

        let mut result = vec![GoldilocksField::from_canonical_u8(0); 8];

        result[..4].copy_from_slice(&r);
        result[4..(4 + 4)].copy_from_slice(&s);

        result
    }
}

impl Decode for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 64 {
            return Err(Error::DecodeError(format!(
                "Invalid length for Signature: expected 64, got {}",
                bytes.len()
            )));
        }
        let r = Hash::from_bytes(&bytes[..32])?;
        let s = Hash::from_bytes(&bytes[32..])?;
        Ok(Self { r, s })
    }
}

impl DecodeFields for Signature {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 8 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for Signature: expected 8, got {}",
                fields.len()
            )));
        }

        let r = Hash::from_fields(&fields[0..4])?;
        let s = Hash::from_fields(&fields[4..8])?;

        Ok(Self { r, s })
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;

    crate::test_encode_bytes!(Signature);
    crate::test_encode_fields!(Signature);
}
