use crate::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fission {
    pub input: Hash,
    pub outputs: [Hash; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fusion {
    pub inputs: [Hash; 2],
    pub output: Hash,
}
