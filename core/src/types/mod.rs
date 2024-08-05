use serde::{Deserialize, Serialize};

mod hash;
mod manifest;

pub use self::{hash::*, manifest::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Transaction {
    pub manifest: Manifest,
    pub inputs: Inputs,
    pub outputs: Outputs,
    #[serde(with = "serde_bytes")]
    pub data: [u8; 256 * 8],
    pub assets: [Hash; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Inputs {
    pub parents: [Hash; 4],
    pub indexes: [u8; 4],
    pub asset_ids: [u8; 4],
    pub amounts: [u64; 4],
    pub program_id: [Hash; 4],
    pub data: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Outputs {
    pub asset_ids: [u8; 4],
    pub amounts: [u64; 4],
    pub program_id: [Hash; 4],
    pub data: [u8; 4],
}
