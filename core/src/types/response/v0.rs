use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "r")]
pub enum Response {
    #[serde(rename = "transaction")]
    Transaction {
        #[serde(rename = "s")]
        outputs: Vec<Blinded<Signature>>,
    },
}
