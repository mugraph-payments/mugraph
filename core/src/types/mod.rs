use serde::{Deserialize, Serialize};

mod builder;
mod datum;
mod hash;
mod manifest;
mod note;

pub use self::{builder::*, datum::*, hash::*, manifest::*, note::*};

pub const MAX_ATOMS: usize = 8;
pub const MAX_INPUTS: usize = 4;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Blob {
    pub asset_id_indexes: [u8; MAX_ATOMS],
    pub amounts: [u64; MAX_ATOMS],
    pub asset_ids: [Hash; MAX_INPUTS],
    pub nonces: [Hash; MAX_ATOMS],
    pub parent_ids: [Hash; MAX_ATOMS],
    pub program_ids: [Hash; MAX_ATOMS],
    pub data: [Datum; MAX_ATOMS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Transaction {
    pub manifest: Manifest,
    pub blob: Blob,
}
