use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

pub mod v0;

pub use v0::Request as V0Request;

#[derive(Debug, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "n")]
pub enum Request {
    #[serde(rename = "v0")]
    V0(v0::Request),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_serialization() {
        let request = Request::V0(v0::Request::Transaction(
            crate::types::Transaction::default(),
        ));

        let expected = json!({
            "n": "v0",
            "m": "transaction",
            "p": {
                "a": [],
                "a_": [],
                "s": [],
            }
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }
}
