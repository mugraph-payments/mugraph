use crate::{Hash, Note};
use serde::{Deserialize, Serialize};

pub type Nullifier = Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Split {
    pub input: Note,
    pub amount: u64,
}

pub struct Join {
    pub inputs: [Note; 2],
}
