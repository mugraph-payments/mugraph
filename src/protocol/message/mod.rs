use circuit::{Seal, Sealable, F};
use serde::{Deserialize, Serialize};

use crate::{protocol::*, Error};

mod append;

pub use append::{Append, Circuit as AppendCircuit};

pub trait ToMessage: Sealable {
    fn method() -> Method;

    fn to_message(&self) -> Result<Message, Error> {
        Ok(Message {
            method: Self::method(),
            program_id: Hash::from_fields(
                &Self::circuit_data().verifier_only.circuit_digest.elements,
            )?,
            seal: self.seal()?,
            payload: Payload {
                inputs: vec![],
                outputs: vec![],
            },
        })
    }
}
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

impl EncodeFields for Payload {
    fn as_fields(&self) -> Vec<F> {
        self.inputs
            .iter()
            .map(|x| x.as_fields())
            .chain(self.outputs.iter().map(|x| x.as_fields()))
            .flatten()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Message {
    pub method: Method,
    pub program_id: Hash,
    pub seal: Seal,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SignedMessage {
    pub message: Message,
    pub signatures: Vec<Signature>,
}
