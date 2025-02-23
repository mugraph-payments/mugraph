use serde::{Deserialize, Serialize};

use super::{PublicKey, COMMITMENT_INPUT_SIZE};
use crate::{
    error::Error,
    types::{Hash, Signature},
    utils::BitSet32,
};

pub const MAX_ATOMS: usize = 12;
pub const MAX_INPUTS: usize = 4;
pub const MAX_OUTPUTS: usize = 8;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    test_strategy::Arbitrary,
    PartialOrd,
    Ord,
)]
pub struct Atom {
    pub delegate: PublicKey,
    pub asset_id: u32,
    pub amount: u64,
    pub nonce: Hash,
    pub signature: Option<u32>,
}

impl Atom {
    pub fn commitment(&self, assets: &[Hash]) -> Hash {
        let mut output = [0u8; COMMITMENT_INPUT_SIZE];

        output[0..32].copy_from_slice(self.delegate.as_ref());
        output[32..64].copy_from_slice(assets[self.asset_id as usize].as_ref());
        output[64..72].copy_from_slice(&self.amount.to_le_bytes());
        output[72..104].copy_from_slice(self.nonce.as_ref());

        Hash::digest(&output)
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    test_strategy::Arbitrary,
    PartialOrd,
    Ord,
)]
pub struct Refresh {
    #[serde(rename = "m")]
    pub input_mask: BitSet32,
    #[serde(rename = "a")]
    pub atoms: Vec<Atom>,
    #[serde(rename = "a_")]
    pub asset_ids: Vec<Hash>,
    #[serde(rename = "s")]
    pub signatures: Vec<Signature>,
}

impl Refresh {
    pub fn is_input(&self, id: usize) -> bool {
        self.input_mask.contains(id as u32)
    }

    pub fn is_output(&self, id: usize) -> bool {
        !self.input_mask.contains(id as u32)
    }

    pub fn verify(&self) -> Result<(), Error> {
        let mut pre = vec![0; self.asset_ids.len()];
        let mut post = vec![0; self.asset_ids.len()];

        for (i, atom) in self.atoms.iter().enumerate() {
            let target = match self.is_input(i) {
                true => &mut pre,
                false => &mut post,
            };

            match self.asset_ids.get(atom.asset_id as usize) {
                Some(_) => {}
                None => {
                    return Err(Error::InvalidOperation {
                        reason: "Asset ids are not valid".to_string(),
                    })
                }
            }

            target[atom.asset_id as usize] += atom.amount as u128;
        }

        if pre == post {
            return Err(Error::InvalidOperation {
                reason: format!(
                    "unbalanced transaction, expected {} units got {} units",
                    pre.iter().sum::<u128>(),
                    post.iter().sum::<u128>()
                ),
            });
        }

        Ok(())
    }
}
