use serde::{Deserialize, Serialize};

use super::Signature;
use crate::{types::Hash, util::BitSet8};

pub const MAX_ATOMS: usize = 8;
pub const MAX_INPUTS: usize = 4;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Transaction {
    #[serde(rename = "m")]
    pub input_mask: BitSet8,
    #[serde(rename = "a_")]
    pub asset_id_indexes: Vec<u8>,
    #[serde(rename = "n")]
    pub amounts: Vec<u64>,
    #[serde(rename = "a")]
    pub asset_ids: Vec<Hash>,
    #[serde(rename = "c")]
    pub commitments: Vec<Hash>,
    #[serde(rename = "s")]
    pub signatures: Vec<Signature>,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        assert_eq!(128, size_of::<Transaction>());
        assert_eq!(8, align_of::<Transaction>());
    }

    #[proptest]
    // Tests if a Transaction struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Transaction) {
        prop_assert_eq!(size_of::<Transaction>(), size_of_val(&note));
    }
}
