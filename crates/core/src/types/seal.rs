use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Sealed<T> {
    #[n(1)]
    pub inner: T,
    #[n(2)]
    #[serde(with = "serde_bytes")]
    pub seal: [u8; 256],
}
