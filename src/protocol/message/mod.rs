use serde::{Deserialize, Serialize};

use crate::protocol::*;

mod append;

pub use append::Append;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Method {
    #[serde(rename = "mu.v1.append")]
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Payload {
    pub inputs: Vec<Signature>,
    pub outputs: Vec<BlindedValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Message {
    pub method: Method,
    pub program_id: Hash,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SignedMessage {
    pub message: Message,
    pub signatures: Vec<Signature>,
}
