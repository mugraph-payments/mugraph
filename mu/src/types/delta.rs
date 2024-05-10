use std::hash::Hash as StdHash;

use blake3::Hash;
use ed25519_dalek::{Signature, VerifyingKey};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::Theta;

pub type ZKProof = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delta {
    /// The public spend keys for each of the input assets
    pub inputs: Vec<VerifyingKey>,
    /// The public spend keys for each of the new created assets
    pub outputs: Vec<VerifyingKey>,
    /// The proofs for each of the programs required to accept this Delta
    pub proofs: IndexMap<Hash, ZKProof>,
    /// The signatures for each of the input assets
    pub signatures: Vec<Signature>,
}

impl Delta {
    pub fn new(
        inputs: Vec<Theta>,
        outputs: Vec<VerifyingKey>,
        proofs: IndexMap<Hash, ZKProof>,
    ) -> Self {
        let inputs = inputs.into_iter().map(|x| x.public_spend_key).collect();
        let signatures = inputs.iter();

        Delta {
            inputs,
            outputs,
            proofs,
            signatures: vec![],
        }
    }
}

impl StdHash for Delta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let inputs = self
            .inputs
            .iter()
            .cloned()
            .sorted_by(|a, b| a.as_bytes().cmp(b.as_bytes()))
            .collect_vec();
        let outputs = self
            .outputs
            .iter()
            .cloned()
            .sorted_by(|a, b| a.as_bytes().cmp(b.as_bytes()))
            .collect_vec();
        let signatures = self
            .signatures
            .iter()
            .cloned()
            .map(|x| x.to_bytes())
            .sorted_by(|a, b| a.to_vec().cmp(&b.to_vec()))
            .collect_vec();
        let proofs = self
            .proofs
            .iter()
            .map(|(k, v)| (k, v.clone()))
            .sorted_by(|(&ka, _), (kb, _)| ka.as_bytes().cmp(kb.as_bytes()))
            .collect_vec();

        inputs.hash(state);
        outputs.hash(state);
        proofs.hash(state);
        signatures.hash(state);
    }
}
