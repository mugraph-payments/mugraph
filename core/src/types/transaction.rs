use serde::{Deserialize, Serialize};

use crate::{types::Hash, util::BitSet8};

pub const MAX_ATOMS: usize = 8;
pub const MAX_INPUTS: usize = 4;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Transaction {
    pub input_mask: BitSet8,
    pub asset_id_indexes: [u8; MAX_ATOMS],
    pub amounts: [u64; MAX_ATOMS],
    pub asset_ids: [Hash; MAX_INPUTS],
    pub nonces: [Hash; MAX_ATOMS],
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        assert_eq!(464, size_of::<Transaction>());
        assert_eq!(8, align_of::<Transaction>());
    }

    #[proptest]
    // Tests if a Transaction struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Transaction) {
        prop_assert_eq!(size_of::<Transaction>(), size_of_val(&note));
    }
}
