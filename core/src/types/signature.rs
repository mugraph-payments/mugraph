use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Signature {
    pub r: Hash,
    pub s: Hash,
}
