use minicbor::{Decode, Encode};

pub use crate::types::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Reaction {
    #[n(0)]
    pub nullifiers: Vec<Hash>,
}
