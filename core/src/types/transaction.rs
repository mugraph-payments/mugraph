use serde::{Deserialize, Serialize};

use crate::{
    types::{Hash, Manifest},
    util::BitSet8,
};

pub const MAX_ATOMS: usize = 8;
pub const MAX_INPUTS: usize = 4;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Blob {
    pub input_mask: BitSet8,
    pub asset_id_indexes: [u8; MAX_ATOMS],
    pub amounts: [u64; MAX_ATOMS],
    pub asset_ids: [Hash; MAX_INPUTS],
    pub nonces: [Hash; MAX_ATOMS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Transaction {
    pub manifest: Manifest,
    pub blob: Blob,
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        assert_eq!(464, size_of::<Blob>());
        assert_eq!(8, align_of::<Blob>());
    }

    #[proptest]
    // Tests if a Blob struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Blob) {
        prop_assert_eq!(size_of::<Blob>(), size_of_val(&note));
    }
}
