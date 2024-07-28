use crate::{Hash, Note};
use serde::{Deserialize, Serialize};

pub type Nullifier = Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestSpend {
    pub input: Note,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Spend {
    pub input: Hash,
    pub outputs: [Hash; 2],
}
