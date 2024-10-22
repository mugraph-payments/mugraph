use plonky2::{hash::poseidon::PoseidonHash, plonk::config::Hasher};

use super::F;
use crate::{protocol::Hash, Error};

pub trait Encode {
    fn as_bytes(&self) -> Vec<u8>;
}

pub trait EncodeFields {
    fn as_fields(&self) -> Vec<F>;

    fn hash(&self) -> Hash {
        PoseidonHash::hash_no_pad(&self.as_fields()).into()
    }
}

pub trait Decode: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
}

pub trait DecodeFields: Sized {
    fn from_fields(bytes: &[F]) -> Result<Self, Error>;
}
