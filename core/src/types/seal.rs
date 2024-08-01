use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Sealed<T> {
    #[n(0)]
    pub inner: T,
    #[n(1)]
    pub hash: Hash,
    #[n(2)]
    #[serde(with = "serde_bytes")]
    pub seal: [u8; 256],
}
