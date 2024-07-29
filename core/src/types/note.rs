use crate::{Hash, Result, Signature};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Note {
    pub asset_id: Hash,
    pub amount: u64,
    pub nullifier: Signature,
}

impl Note {
    pub fn as_bytes(&self) -> [u8; 104] {
        let mut bytes = [0u8; 104];
        bytes[..32].copy_from_slice(&*self.asset_id);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&self.nullifier.to_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8; 104]) -> Result<Self> {
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
