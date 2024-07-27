use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub asset_id: [u8; 32],
    pub amount: u64,
    pub nullifier: [u8; 32],
}

impl Note {
    pub fn as_bytes(&self) -> [u8; 72] {
        let mut bytes = [0u8; 72];
        bytes[..32].copy_from_slice(&self.asset_id);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&self.nullifier);
        bytes
    }
}
