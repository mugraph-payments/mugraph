use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod hash;
mod manifest;
mod note;
mod operation;
mod reaction;
mod sealed;

pub use self::{hash::*, manifest::*, note::*, operation::*, reaction::*, sealed::*};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Request<T> {
    #[n(0)]
    pub manifest: Manifest,
    #[n(1)]
    pub data: T,
}
