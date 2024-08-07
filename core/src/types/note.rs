use serde::{Deserialize, Serialize};

use crate::types::*;

pub const COMMITMENT_INPUT_SIZE: usize = 72;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Note {
    pub parent_id: Hash,
    pub asset_id: Hash,
    pub nonce: Hash,
    pub amount: u64,
}

impl Note {
    pub fn commitment(&self) -> Hash {
        let mut output = [0u8; COMMITMENT_INPUT_SIZE];

        output[0..32].copy_from_slice(self.asset_id.as_ref());
        output[32..40].copy_from_slice(self.amount.to_le_bytes().as_ref());
        output[40..COMMITMENT_INPUT_SIZE].copy_from_slice(self.nonce.as_ref());

        Hash::digest(&output)
    }
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        assert_eq!(size_of::<Note>(), 104);
        assert_eq!(align_of::<Note>(), 8);
    }

    #[proptest]
    // Tests if a Note struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Note) {
        prop_assert_eq!(size_of::<Note>(), size_of_val(&note));
    }

    #[proptest]
    fn test_commitment(note: Note) {
        let expected = [
            note.asset_id.as_ref(),
            note.amount.to_le_bytes().as_ref(),
            note.nonce.as_ref(),
        ]
        .concat();

        prop_assert_eq!(Hash::digest(&expected), note.commitment());
    }
}
