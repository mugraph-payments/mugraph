use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Note {
    pub asset_id: Hash,
    pub amount: u64,
    pub nullifier: Signature,
}

impl SerializeBytes for Note {
    const SIZE: usize = Hash::SIZE + u64::SIZE + Signature::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        self.asset_id.to_slice(&mut out[..Hash::SIZE]);
        self.amount
            .to_le_bytes()
            .copy_from_slice(&mut out[Hash::SIZE..Hash::SIZE + u64::SIZE]);
        self.nullifier.to_slice(&mut out[Hash::SIZE + u64::SIZE..]);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        if input.len() < Self::SIZE {
            return Err(Error::FailedDeserialization);
        }

        Ok(Self {
            asset_id: Hash::from_slice(&input[..Hash::SIZE])?,
            amount: u64::from_le_bytes(
                input[Hash::SIZE..Hash::SIZE + u64::SIZE]
                    .try_into()
                    .unwrap(),
            ),
            nullifier: Signature::from_slice(&input[Hash::SIZE + u64::SIZE..])?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct BlindedNote {
    pub asset_id: Hash,
    pub amount: u64,
    pub secret: Hash,
}

impl BlindedNote {
    pub const SIZE: usize = 72;

    pub fn unblind(self, signature: Signature) -> Note {
        Note {
            asset_id: self.asset_id,
            amount: self.amount,
            nullifier: signature,
        }
    }
}

impl SerializeBytes for BlindedNote {
    const SIZE: usize = Hash::SIZE + u64::SIZE + Hash::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        self.asset_id.to_slice(&mut out[..Hash::SIZE]);
        self.amount
            .to_le_bytes()
            .copy_from_slice(&mut out[Hash::SIZE..Hash::SIZE + u64::SIZE]);
        self.secret.to_slice(&mut out[Hash::SIZE + u64::SIZE..]);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        if input.len() < Self::SIZE {
            return Err(Error::FailedDeserialization);
        }

        Ok(Self {
            asset_id: Hash::from_slice(&input[..Hash::SIZE])?,
            amount: u64::from_le_bytes(
                input[Hash::SIZE..Hash::SIZE + u64::SIZE]
                    .try_into()
                    .unwrap(),
            ),
            secret: Hash::from_slice(&input[Hash::SIZE + u64::SIZE..])?,
        })
    }
}
