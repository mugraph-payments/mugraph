#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub asset_id: [u8; 32],
    pub amount: u64,
    pub nullifier: [u8; 32],
}

pub struct Transaction<const M: usize, const N: usize> {
    pub inputs: [Note; M],
    pub outputs: [Note; N],
}
