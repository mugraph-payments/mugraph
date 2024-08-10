use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Verification {
    pub server_key: PublicKey,
    pub hash: Hash,
}

pub fn verify(note: &Note) {
    env::commit(&Verification {
        server_key: note.delegate,
        hash: note.commitment(),
    });
}
