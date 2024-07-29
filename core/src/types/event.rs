use crate::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fission {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fusion {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}
