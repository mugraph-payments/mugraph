use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Note {
    #[n(0)]
    pub asset_id: Hash,
    #[n(1)]
    pub amount: u64,
    #[n(2)]
    pub program_id: Option<Hash>,
    #[cbor(n(4), with = "minicbor::bytes")]
    pub datum: Option<Vec<u8>>,
}
