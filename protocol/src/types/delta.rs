use std::hash::Hash as StdHash;

use blake3::Hash;
use ed25519_dalek::{Signature, Signer, VerifyingKey};
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

fn sign(theta: &Theta) -> Signature {
    theta
        .spend_key
        .sign(blake3::hash(&theta.as_bytes()).as_bytes())
}

impl Delta {
    pub fn new(
        inputs: Vec<Theta>,
        outputs: Vec<VerifyingKey>,
        proofs: IndexMap<Hash, ZKProof>,
    ) -> Self {
        let signatures = inputs.iter().map(sign).collect_vec();

        Delta {
            inputs: inputs.iter().map(|x| x.public_spend_key).collect_vec(),
            outputs,
            proofs,
            signatures,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let inputs = self
            .inputs
            .iter()
            .map(|x| x.to_bytes().to_vec())
            .sorted_by(|a, b| a.to_vec().cmp(&b.to_vec()))
            .collect_vec();
        let outputs = self
            .outputs
            .iter()
            .map(|x| x.to_bytes().to_vec())
            .sorted_by(|a, b| a.to_vec().cmp(&b.to_vec()))
            .collect_vec();
        let signatures = self
            .signatures
            .iter()
            .map(|x| x.to_bytes().to_vec())
            .sorted_by(|a, b| a.to_vec().cmp(&b.to_vec()))
            .collect_vec();
        let proofs = self
            .proofs
            .iter()
            .map(|(k, v)| [k.as_bytes().to_vec(), v.as_bytes().to_vec()].concat())
            .sorted_by(|a, b| a.to_vec().cmp(&b.to_vec()))
            .collect_vec();

        [inputs, outputs, signatures, proofs]
            .concat()
            .into_iter()
            .flatten()
            .collect_vec()
    }
}

impl StdHash for Delta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}
