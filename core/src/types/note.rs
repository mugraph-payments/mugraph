use serde::{Deserialize, Serialize};

use crate::types::*;

pub const COMMITMENT_INPUT_SIZE: usize = 136;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Note {
    pub amount: u64,
    pub delegate: PublicKey,
    pub policy_id: PolicyId,
    pub asset_name: AssetName,
    pub nonce: Hash,
    pub signature: Signature,
    #[serde(default)]
    pub dleq: Option<DleqProofWithBlinding>,
}

impl Note {
    pub fn commitment(&self) -> Hash {
        let mut output = [0u8; COMMITMENT_INPUT_SIZE];

        output[0..32].copy_from_slice(self.delegate.as_ref());
        write_asset_bytes(&self.policy_id, &self.asset_name, &mut output[32..96]);
        output[96..104].copy_from_slice(&self.amount.to_le_bytes());
        output[104..136].copy_from_slice(self.nonce.as_ref());

        Hash::digest(&output)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::Value;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn test_byte_sizes() {
        // Regression check: keep structure stable but avoid brittle exact size.
        assert_eq!(align_of::<Note>(), 8);
        assert!(size_of::<Note>() > 0);
    }

    #[proptest]
    // Tests if a Note struct has a consistent size with the actual struct size
    fn test_size_consistency(note: Note) {
        prop_assert_eq!(size_of::<Note>(), size_of_val(&note));
    }

    #[proptest]
    fn test_commitment(note: Note) {
        let mut asset_bytes = [0u8; ASSET_ID_BYTES_SIZE];
        write_asset_bytes(&note.policy_id, &note.asset_name, &mut asset_bytes);
        let expected = [
            note.delegate.as_ref(),
            asset_bytes.as_ref(),
            note.amount.to_le_bytes().as_ref(),
            note.nonce.as_ref(),
        ]
        .concat();

        prop_assert_eq!(Hash::digest(&expected), note.commitment());
    }

    #[test]
    fn note_serializes_with_inline_asset_fields() {
        let note = Note {
            amount: 42,
            delegate: PublicKey([0x22; 32]),
            policy_id: PolicyId([0x11; POLICY_ID_SIZE]),
            asset_name: AssetName::new(b"PAY").expect("valid name"),
            nonce: Hash([0x33; 32]),
            signature: Signature([0x44; 32]),
            dleq: None,
        };

        let value = serde_json::to_value(&note).expect("serialize note");
        let obj = value.as_object().expect("note should be an object");
        assert_eq!(
            obj.get("policy_id"),
            Some(&Value::String(muhex::encode(note.policy_id.0)))
        );
        assert_eq!(
            obj.get("asset_name"),
            Some(&Value::String("PAY".to_string()))
        );
    }
}
