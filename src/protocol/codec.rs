use plonky2::{hash::poseidon::PoseidonHash, plonk::config::Hasher};

use super::circuit::{Field, F};
use crate::{protocol::Hash, Error};

pub const MAGIC_PREFIX: [u8; 16] = *b"mugraph.v1.ecash";
pub const MAGIC_PREFIX_FIELDS: [u64; 2] = [3344046287156114797, 7526466481793413494];

pub trait Encode {
    fn as_bytes(&self) -> Vec<u8>;

    fn as_bytes_with_prefix(&self) -> Vec<u8> {
        [MAGIC_PREFIX.to_vec(), self.as_bytes()].concat()
    }
}

pub trait EncodeFields {
    fn as_fields(&self) -> Vec<F>;

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

    fn hash(&self) -> Hash {
        PoseidonHash::hash_no_pad(&self.as_fields_with_prefix()).into()
    }
}

impl<T: EncodeFields> EncodeFields for [T] {
    fn as_fields(&self) -> Vec<F> {
        self.iter().flat_map(|x| x.as_fields()).collect()
    }
}

pub trait Decode: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
}

pub trait DecodeFields: Sized {
    fn from_fields(bytes: &[F]) -> Result<Self, Error>;
}
