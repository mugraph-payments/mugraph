use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{Blinded, Hash, Signature};

/// Schnorr-style proof that the same secret key was used for both the
/// long-term public key and a blind signature response.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Arbitrary,
    PartialOrd,
    Ord,
    ::core::hash::Hash,
)]
pub struct DleqProof {
    #[serde(rename = "e")]
    pub challenge: Hash,
    #[serde(rename = "z")]
    pub response: Hash,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Arbitrary,
    PartialOrd,
    Ord,
    ::core::hash::Hash,
)]
pub struct DleqProofWithBlinding {
    #[serde(flatten)]
    pub proof: DleqProof,
    #[serde(rename = "r")]
    pub blinding_factor: Hash,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Arbitrary,
    PartialOrd,
    Ord,
    ::core::hash::Hash,
)]
pub struct BlindSignature {
    #[serde(rename = "c")]
    pub signature: Blinded<Signature>,
    #[serde(rename = "p")]
    pub proof: DleqProof,
}
