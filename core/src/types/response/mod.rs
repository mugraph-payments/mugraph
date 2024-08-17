use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

pub mod v0;

pub use v0::Response as V0Response;

#[derive(Debug, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "n")]
pub enum Response {
    #[serde(rename = "v0")]
    V0(v0::Response),
}

impl From<v0::Response> for Response {
    fn from(response: v0::Response) -> Self {
        Self::V0(response)
    }
}
