use crate::{Error, Hash, Result, Signature};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Note {
    pub asset_id: Hash,
    pub amount: u64,
    pub nullifier: Signature,
}

impl Note {
    pub const SIZE: usize = 104;

    pub fn to_slice(&self, out: &mut [u8]) {
        out[..32].copy_from_slice(&*self.asset_id);
        out[32..40].copy_from_slice(&self.amount.to_le_bytes());
        out[40..].copy_from_slice(&self.nullifier.to_bytes());
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(Error::FailedDeserialization);
        }

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

    pub fn digest(&self) -> Hash {
        let mut buf = [0u8; Self::SIZE];
        self.to_slice(&mut buf);
        Hash::digest(&buf).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlindedNote {
    pub asset_id: Hash,
    pub amount: u64,
    pub secret: Hash,
}

impl BlindedNote {
    pub const SIZE: usize = 72;

    pub fn to_slice(&self, out: &mut [u8]) {
        out[..32].copy_from_slice(&*self.asset_id);
        out[32..40].copy_from_slice(&self.amount.to_le_bytes());
        out[40..Self::SIZE].copy_from_slice(&*self.secret);
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(Error::FailedDeserialization);
        }

        let mut asset_id = Hash::default();
        let mut blinded_secret = Hash::default();
        let mut amount = [0u8; 8];

        asset_id.copy_from_slice(&bytes[..32]);
        amount.copy_from_slice(&bytes[32..40]);
        blinded_secret.copy_from_slice(&bytes[40..Self::SIZE]);

        Ok(Self {
            asset_id,
            amount: u64::from_le_bytes(amount),
            secret: blinded_secret,
        })
    }

    pub fn digest(&self) -> Hash {
        let mut buf = [0u8; Self::SIZE];
        self.to_slice(&mut buf);
        Hash::digest(&buf).unwrap()
    }

    pub fn unblind(self, signature: Signature) -> Note {
        Note {
            asset_id: self.asset_id,
            amount: self.amount,
            nullifier: signature,
        }
    }
}
