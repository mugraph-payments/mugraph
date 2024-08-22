use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{PublicKey, Signature};
use crate::types::Hash;

pub const MAX_ATOMS: usize = 8;
pub const MAX_INPUTS: usize = 4;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Atom {
    pub delegate: PublicKey,
    pub asset_id: u32,
    pub amount: u64,
    pub nonce: Hash,
    pub signature: Option<u32>,
}

impl Atom {
    pub fn is_input(&self) -> bool {
        self.signature.is_none()
    }
}

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, test_strategy::Arbitrary,
)]
pub struct Transaction {
    #[serde(rename = "a")]
    pub atoms: Vec<Atom>,
    #[serde(rename = "a_")]
    pub asset_ids: Vec<Hash>,
    #[serde(rename = "s")]
    pub signatures: Vec<Signature>,
}

impl Transaction {
    pub fn is_balanced(&self) -> bool {
        let mut pre_balances = BTreeMap::new();
        let mut post_balances = BTreeMap::new();

        for atom in self.atoms.iter() {
            let target = match atom.is_input() {
                true => &mut pre_balances,
                false => &mut post_balances,
            };

            let asset_id = match self.asset_ids.get(atom.asset_id as usize) {
                Some(a) => a,
                None => return false,
            };

            target
                .entry(asset_id)
                .and_modify(|x| *x += atom.amount as u128)
                .or_insert(atom.amount as u128);
        }

        pre_balances == post_balances
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::Transaction;

    #[proptest]
    fn test_transaction_balance(tx: Transaction) {
        let mut balance_difference = BTreeMap::new();

        for atom in &tx.atoms {
            if let Some(asset_id) = tx.asset_ids.get(atom.asset_id as usize) {
                let change = if atom.is_input() {
                    atom.amount as i128
                } else {
                    -(atom.amount as i128)
                };
                *balance_difference.entry(asset_id).or_insert(0) += change;
            } else {
                // Invalid asset_id
                prop_assert!(!tx.is_balanced());
                return Ok(());
            }
        }

        let is_balanced = balance_difference.values().all(|&balance| balance == 0);
        prop_assert_eq!(tx.is_balanced(), is_balanced);
    }
}
