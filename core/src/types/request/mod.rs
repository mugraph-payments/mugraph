use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

pub mod v0;

#[derive(Debug, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "n")]
pub enum Request {
    #[serde(rename = "v0")]
    V0(v0::Request),
}

impl From<v0::Request> for Request {
    fn from(value: v0::Request) -> Self {
        Self::V0(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::types::{Refresh, V0Request};

    #[test]
    fn test_serialization() {
        let request: Request = V0Request::Refresh(Refresh::default()).into();

        let expected = json!({
            "n": "v0",
            "m": "transaction",
            "p": {
                "m": 0,
                "a": [],
                "a_": [],
                "s": [],
            }
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }
}
