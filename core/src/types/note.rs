use crate::{Hash, Result, Signature};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Note {
    pub asset_id: Hash,
    pub amount: u64,
    pub nullifier: Signature,
}

impl Note {
    pub const SIZE: usize = 104;

    pub fn as_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[..32].copy_from_slice(&*self.asset_id);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&self.nullifier.to_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let mut asset_id = Hash::default();
        let mut amount = [0u8; 8];

        asset_id.copy_from_slice(&bytes[..32]);
        amount.copy_from_slice(&bytes[32..40]);

        Ok(Self {
            asset_id,
            amount: u64::from_le_bytes(amount),
            nullifier: Signature::from_bytes(&bytes[40..])?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlindedNote {
    pub asset_id: Hash,
    pub amount: u64,
    pub secret: Hash,
}

impl BlindedNote {
    pub fn as_bytes(&self) -> [u8; 72] {
        let mut bytes = [0u8; 72];
        bytes[..32].copy_from_slice(&*self.asset_id);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&*self.secret);
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 72]) -> Result<Self> {
        let mut asset_id = Hash::default();
        let mut blinded_secret = Hash::default();
        let mut amount = [0u8; 8];

        asset_id.copy_from_slice(&bytes[..32]);
        amount.copy_from_slice(&bytes[32..40]);
        blinded_secret.copy_from_slice(&bytes[40..]);

        Ok(Self {
            asset_id,
            amount: u64::from_le_bytes(amount),
            secret: blinded_secret,
        })
    }

    pub fn unblind(self, signature: Signature) -> Note {
        Note {
            asset_id: self.asset_id,
            amount: self.amount,
            nullifier: signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_to_from_bytes() {
        let original_note = Note {
            asset_id: Hash::from([1u8; 32]),
            amount: 1000,
            nullifier: Signature::from_bytes(&[2u8; 64]).unwrap(),
        };

        let bytes = original_note.as_bytes();
        assert_eq!(bytes.len(), Note::SIZE);

        let reconstructed_note = Note::from_bytes(&bytes).unwrap();
        assert_eq!(original_note, reconstructed_note);
    }

    #[test]
    fn test_blinded_note_to_from_bytes() {
        let original_blinded_note = BlindedNote {
            asset_id: Hash::from([3u8; 32]),
            amount: 2000,
            secret: Hash::from([4u8; 32]),
        };

        let bytes = original_blinded_note.as_bytes();
        assert_eq!(bytes.len(), 72);

        let reconstructed_blinded_note = BlindedNote::from_bytes(&bytes).unwrap();
        assert_eq!(original_blinded_note, reconstructed_blinded_note);
    }

    #[test]
    fn test_note_byte_representation() {
        let note = Note {
            asset_id: Hash::from([5u8; 32]),
            amount: 3000,
            nullifier: Signature::from_bytes(&[6u8; 64]).unwrap(),
        };

        let bytes = note.as_bytes();

        assert_eq!(&bytes[..32], &[5u8; 32]);
        assert_eq!(&bytes[32..40], &3000u64.to_le_bytes());
        assert_eq!(&bytes[40..], &[6u8; 64]);
    }

    #[test]
    fn test_blinded_note_byte_representation() {
        let blinded_note = BlindedNote {
            asset_id: Hash::from([7u8; 32]),
            amount: 4000,
            secret: Hash::from([8u8; 32]),
        };

        let bytes = blinded_note.as_bytes();

        assert_eq!(&bytes[..32], &[7u8; 32]);
        assert_eq!(&bytes[32..40], &4000u64.to_le_bytes());
        assert_eq!(&bytes[40..], &[8u8; 32]);
    }
}
