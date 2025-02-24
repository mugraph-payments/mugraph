use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "r")]
pub enum Response {
    #[serde(rename = "refresh")]
    Transaction {
        #[serde(rename = "s")]
        outputs: Vec<Blinded<Signature>>,
    },
    #[serde(rename = "emit")]
    Emit(Note),
    #[serde(rename = "error")]
    Error { reason: String },
}
