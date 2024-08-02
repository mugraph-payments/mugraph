use std::collections::HashMap;

use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::Hash;

#[derive(Debug, Clone, Default, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct CompressedTransaction {
    #[n(0)]
    pub asset_ids: Vec<Hash>,
    #[n(1)]
    pub pre_balances: HashMap<u32, u64>,
    #[n(2)]
    pub post_balances: HashMap<u32, u64>,
}
