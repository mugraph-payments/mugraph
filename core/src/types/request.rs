use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::{Hash, Signature};

#[derive(Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Request {
    #[serde(rename = "s")]
    Simple(SimpleRequest),
    //#[serde(rename = "z")]
    //Zk(ZKRequest),
}

#[derive(Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "a")]
    pub asset_id: Hash,
    #[serde(rename = "n")]
    pub amount: u64,
    #[serde(rename = "s")]
    pub signature: Signature,
}

#[derive(Serialize, Deserialize)]
pub struct SimpleRequest {
    #[serde(rename = "i")]
    pub inputs: Vec<Input>,
}
