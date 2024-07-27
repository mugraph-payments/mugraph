use crate::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Swap {
    pub inputs: [Hash; 8],
    pub outputs: [Hash; 8],
}
