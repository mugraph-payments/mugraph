use crate::protocol::*;
use mucodec::Bytes;

mod append;

pub use append::{Append};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    pub inputs: Vec<Signature>,
    pub outputs: Vec<BlindedValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub method: Method,
    pub program_id: Hash,
    pub seal: Bytes<1024>,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessage {
    pub message: Message,
    pub signatures: Vec<Signature>,
}
