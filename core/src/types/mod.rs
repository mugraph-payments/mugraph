use minicbor::{Decode, Encode, Encoder};
use serde::{Deserialize, Serialize};

mod hash;
mod manifest;

pub use self::{hash::*, manifest::*};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Transaction {
    #[n(0)]
    pub manifest: Manifest,
    #[n(1)]
    pub inputs: Inputs,
    #[n(2)]
    pub outputs: Outputs,
    #[n(3)]
    #[serde(with = "serde_bytes")]
    pub data: [u8; 256 * 8],
    #[n(4)]
    pub assets: [Hash; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Inputs {
    #[n(0)]
    pub parents: [Hash; 4],
    #[n(1)]
    pub indexes: [u8; 4],
    #[n(2)]
    pub asset_ids: [u8; 4],
    #[n(3)]
    pub amounts: [u64; 4],
    #[n(4)]
    pub program_id: [Hash; 4],
    #[n(5)]
    pub data: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Outputs {
    #[n(0)]
    pub asset_ids: [u8; 4],
    #[n(1)]
    pub amounts: [u64; 4],
    #[n(2)]
    pub program_id: [Hash; 4],
    #[n(3)]
    pub data: [u8; 4],
}
