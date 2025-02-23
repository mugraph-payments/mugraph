use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{Hash, Refresh};

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "p")]
pub enum Request {
    #[serde(rename = "transaction")]
    Refresh(Refresh),
    #[serde(rename = "emit")]
    Emit { asset_id: Hash, amount: u64 },
}
