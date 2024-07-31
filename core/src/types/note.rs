use mugraph_derive::SerializeBytes;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Note {
    pub asset_id: Hash,
    pub server_key: PublicKey,
    pub amount: u64,
    pub nullifier: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct BlindedNote {
    pub asset_id: Hash,
    pub server_key: PublicKey,
    pub amount: u64,
    pub secret: Hash,
}

impl BlindedNote {
    pub fn unblind(self, signature: Signature) -> Note {
        Note {
            asset_id: self.asset_id,
            amount: self.amount,
            server_key: self.server_key,
            nullifier: signature,
        }
    }
}
