use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct ProgramSet {
    #[n(0)]
    pub apply: Hash,
    #[n(1)]
    pub compose: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Manifest {
    #[n(0)]
    pub programs: ProgramSet,
}
