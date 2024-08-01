use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, Serialize, Deserialize)]
pub struct Output<P, R> {
    #[n(0)]
    pub public: P,
    #[n(1)]
    pub private: R,
}
