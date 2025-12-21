use serde::{Deserialize, Serialize};

use crate::types::*;

pub const COMMITMENT_INPUT_SIZE: usize = 136;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Note {
    pub amount: u64,
    pub delegate: PublicKey,
    pub asset_id: AssetId,
    pub nonce: Hash,
    pub signature: Signature,
}

impl Note {
    pub fn commitment(&self) -> Hash {
        let mut output = [0u8; COMMITMENT_INPUT_SIZE];

        output[0..32].copy_from_slice(self.delegate.as_ref());
        self.asset_id.write_bytes(&mut output[32..96]);
        output[96..104].copy_from_slice(&self.amount.to_le_bytes());
        output[104..136].copy_from_slice(self.nonce.as_ref());

        Hash::digest(&output)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        assert_eq!(size_of::<Note>(), 168);
        assert_eq!(align_of::<Note>(), 8);
    }

    #[proptest]
    // Tests if a Note struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Note) {
        prop_assert_eq!(size_of::<Note>(), size_of_val(&note));
    }

    #[proptest]
    fn test_commitment(note: Note) {
        let asset_id = note.asset_id.to_bytes();
        let expected = [
            note.delegate.as_ref(),
            asset_id.as_ref(),
            note.amount.to_le_bytes().as_ref(),
            note.nonce.as_ref(),
        ]
        .concat();

        prop_assert_eq!(Hash::digest(&expected), note.commitment());
    }
}
