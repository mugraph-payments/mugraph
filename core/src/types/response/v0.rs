use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{error::Error, types::*};

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "r")]
pub enum Response {
    #[serde(rename = "transaction")]
    Transaction {
        #[serde(rename = "s")]
        outputs: Vec<Blinded<Signature>>,
    },
    #[serde(rename = "error")]
    Error { errors: Vec<Error> },
}
