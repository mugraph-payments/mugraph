use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "p")]
pub enum Request {
    #[serde(rename = "transaction")]
    Transaction(crate::types::Transaction),
}
