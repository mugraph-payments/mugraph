use serde::{Deserialize, Serialize};

use crate::{protocol::*, Error};

mod append;
mod redeem;

pub use append::{Append, Circuit as AppendCircuit};
pub use redeem::{Circuit as RedeemCircuit, Redeem};

pub trait ToMessage: Sealable {
    fn method() -> Method;

    fn to_message(&self) -> Result<Message, Error> {
        Ok(Message {
            method: Self::method(),
            program_id: Hash::from_fields(
                &Self::circuit_data().verifier_only.circuit_digest.elements,
            )?,
            seal: self.seal()?,
            payload: todo!(),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Method {
    #[serde(rename = "mu.v1.redeem")]
    Redeem,
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
    pub seal: Seal,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SignedMessage {
    pub message: Message,
    pub signatures: Vec<Signature>,
}
