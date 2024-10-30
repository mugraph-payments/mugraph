use plonky2::{field::types::PrimeField64, hash::poseidon::PoseidonHash, plonk::config::Hasher};

use super::circuit::{Field, F};
use crate::{protocol::Hash, Error};

pub const MAGIC_PREFIX: [u8; 16] = *b"mugraph.v1.ecash";
pub const MAGIC_PREFIX_FIELDS: [u64; 2] = [3344046287156114797, 7526466481793413494];

pub trait Encode {
    fn as_fields(&self) -> Vec<F>;

    #[inline]
    fn field_len(&self) -> usize {
        self.as_fields().len()
    }

    #[inline]
    fn as_fields_with_prefix(&self) -> Vec<F> {
        [
            MAGIC_PREFIX_FIELDS
                .iter()
                .copied()
                .map(F::from_canonical_u64)
                .collect::<Vec<_>>(),
            self.as_fields(),
        ]
        .concat()
    }

    #[inline]
    fn hash(&self) -> Hash {
        PoseidonHash::hash_no_pad(&self.as_fields()).into()
    }

    #[inline]
    fn hash_with_prefix(&self) -> Hash {
        PoseidonHash::hash_no_pad(&self.as_fields_with_prefix()).into()
    }

    #[inline]
    fn as_bytes(&self) -> Vec<u8> {
        fields_to_bytes(&self.as_fields())
    }

    #[inline]
    fn byte_len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    fn as_bytes_with_prefix(&self) -> Vec<u8> {
        fields_to_bytes(&self.as_fields_with_prefix())
    }

    #[inline]
    fn hash_bytes(&self) -> Hash {
        PoseidonHash::hash_no_pad(&bytes_to_field(&self.as_bytes())).into()
    }

    #[inline]
    fn hash_bytes_with_prefix(&self) -> Hash {
        PoseidonHash::hash_no_pad(&bytes_to_field(&self.as_bytes_with_prefix())).into()
    }
}

impl<T: Encode> Encode for [T] {
    #[inline]
    fn as_fields(&self) -> Vec<F> {
        self.iter().flat_map(|x| x.as_fields()).collect()
    }
}

pub trait Decode: Sized {
    fn from_fields(bytes: &[F]) -> Result<Self, Error>;

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Self::from_fields(&bytes_to_field(bytes))
    }
}

fn bytes_to_field(val: &[u8]) -> Vec<F> {
    val.chunks(32)
        .map(|chunk| {
            let mut padded = [0u8; 32];
            padded[..chunk.len()].copy_from_slice(chunk);
            let value = u64::from_le_bytes(padded[..8].try_into().unwrap());
            F::from_canonical_u64(value)
        })
        .collect()
}

fn fields_to_bytes(fields: &[F]) -> Vec<u8> {
    fields
        .iter()
        .flat_map(|&f| {
            let bytes = f.to_canonical_u64().to_le_bytes();
            bytes.to_vec()
        })
        .collect()
}
