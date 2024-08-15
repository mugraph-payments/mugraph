use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Request {
    #[serde(rename = "s")]
    Simple {
        #[serde(rename = "i")]
        inputs: Vec<Input>,
        #[serde(rename = "o")]
        outputs: Vec<Output>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "a")]
    pub asset_id: Hash,
    #[serde(rename = "$")]
    pub amount: u64,
    #[serde(rename = "n")]
    pub nonce: Hash,
    #[serde(rename = "s")]
    pub signature: Signature,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    #[serde(rename = "a")]
    pub asset_id: Hash,
    #[serde(rename = "$")]
    pub amount: u64,
    #[serde(rename = "s")]
    pub commitment: Hash,
}
