#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub asset_id: [u8; 32],
    pub amount: u64,
    pub nullifier: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: [Option<Note>; 8],
    pub outputs: [Option<Note>; 8],
}
