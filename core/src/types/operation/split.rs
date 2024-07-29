use crate::{error::Result, Note, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Split {
    pub server_key: PublicKey,
    pub input: Note,
    pub amount: u64,
}

impl Split {
    pub const SIZE: usize = 32 + Note::SIZE + 8;

    pub fn to_slice(&self, out: &mut [u8; Self::SIZE]) {
        out[..32].copy_from_slice(&self.server_key);
        self.input.to_slice(&mut out[32..32 + Note::SIZE]);
        out[32 + Note::SIZE..].copy_from_slice(&self.amount.to_le_bytes());
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let server_key = PublicKey::try_from(&bytes[..32]).unwrap();
        let input = Note::from_bytes(&bytes[32..32 + Note::SIZE])?;
        let amount = u64::from_le_bytes(bytes[32 + Note::SIZE..].try_into().unwrap());

        Ok(Self {
            server_key,
            input,
            amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Hash, Signature};

    fn create_test_split() -> Split {
        Split {
            server_key: [1u8; 32],
            input: Note {
                asset_id: Hash::from([2u8; 32]),
                amount: 1000,
                nullifier: Signature {
                    r: Hash::from([3u8; 32]),
                    s: Hash::from([4u8; 32]),
                },
            },
            amount: 500,
        }
    }

    #[test]
    fn test_split_to_from_bytes() {
        let original_split = create_test_split();

        // Convert to bytes
        let mut bytes = [0u8; Split::SIZE];
        original_split.to_slice(&mut bytes);

        // Convert back from bytes
        let reconstructed_split = Split::from_bytes(&bytes).unwrap();

        // Compare
        assert_eq!(original_split, reconstructed_split);
    }

    #[test]
    fn test_split_byte_representation() {
        let split = create_test_split();
        let mut bytes = [0u8; Split::SIZE];
        split.to_slice(&mut bytes);

        // Check server_key
        assert_eq!(&bytes[..32], &[1u8; 32]);

        // Check input note
        assert_eq!(&bytes[32..64], &[2u8; 32]); // asset_id
        assert_eq!(&bytes[64..72], &1000u64.to_le_bytes()); // amount
        assert_eq!(&bytes[72..104], &[3u8; 32]); // nullifier.r
        assert_eq!(&bytes[104..136], &[4u8; 32]); // nullifier.s

        // Check amount
        assert_eq!(&bytes[136..], &500u64.to_le_bytes());
    }
}
