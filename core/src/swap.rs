use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Swap {
    pub inputs: [[u8; 32]; 8],
    pub outputs: [[u8; 32]; 8],
}
