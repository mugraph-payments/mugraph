use minicbor::{Decode, Encode, Encoder};
use risc0_zkvm::sha::{Impl, Sha256};
use serde::{Deserialize, Serialize};

use crate::{error::Result, types::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Sealed<T> {
    #[n(0)]
    pub parent: Hash,
    #[n(1)]
    pub index: u8,
    #[n(2)]
    pub data: T,
}

impl<T: Encode<()>> Sealed<T> {
    pub fn hash(&self) -> Result<Hash> {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode(&self)?;

        Ok((*Impl::hash_bytes(&buf)).into())
    }
}
