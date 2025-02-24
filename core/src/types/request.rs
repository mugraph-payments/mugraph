use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{Hash, Refresh};

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "p")]
pub enum Request {
    #[serde(rename = "refresh")]
    Refresh(Refresh),
    #[serde(rename = "emit")]
    Emit { asset_id: Hash, amount: u64 },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::types::{Refresh, Request};

    #[test]
    fn test_serialization() {
        let request: Request = Request::Refresh(Refresh::default());

        let expected = json!({
            "m": "refresh",
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
