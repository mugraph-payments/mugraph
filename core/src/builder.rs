use indexmap::IndexSet;

use crate::{
    error::{Error, Result},
    keyset::split_amount,
    types::{Asset, AssetName, Atom, Hash, Note, PolicyId, Refresh},
    utils::BitSet32,
};

#[derive(Default)]
pub struct RefreshBuilder {
    pub inputs: Vec<Note>,
    pre_balances: Vec<u128>,
    post_balances: Vec<u128>,
    assets: IndexSet<Asset>,
    outputs: Vec<(u32, u64)>,
}

impl RefreshBuilder {
    pub fn new() -> Self {
        Self {
            pre_balances: vec![0; 8],
            post_balances: vec![0; 8],
            ..Default::default()
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        let asset = Asset {
            policy_id: note.policy_id,
            asset_name: note.asset_name,
        };
        match self.assets.get_index_of(&asset) {
            Some(i) => {
                self.pre_balances[i] += note.amount as u128;
            }
            None => {
                self.pre_balances[self.assets.len()] += note.amount as u128;
                self.assets.insert(asset);
            }
        }

        self.inputs.push(note);

        self
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn output(
        mut self,
        policy_id: PolicyId,
        asset_name: AssetName,
        amount: u64,
    ) -> Self {
        let asset = Asset {
            policy_id,
            asset_name,
        };
        let index = match self.assets.get_index_of(&asset) {
            Some(i) => i,
            None => {
                self.assets.insert(asset);
                self.assets.len() - 1
            }
        };

        // Each output atom must be one denomination, so an amount becomes
        // one atom for each bit that is set.
        self.post_balances[index] += amount as u128;
        for part in split_amount(amount) {
            self.outputs.push((index as u32, part));
        }

        self
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn build(self) -> Result<Refresh> {
        let mut atoms = Vec::new();
        let mut signatures = Vec::new();
        let mut input_mask = BitSet32::new();
        let delegate = self.inputs[0].delegate;

        for (index, note) in self.inputs.into_iter().enumerate() {
            input_mask.insert(index as u32);

            let asset_id = match self.assets.get_index_of(&Asset {
                policy_id: note.policy_id,
                asset_name: note.asset_name,
            }) {
                Some(a) => a as u32,
                None => {
                    return Err(Error::InvalidOperation {
                        reason: "Missing asset_id for iput".to_string(),
                    });
                }
            };

            atoms.push(Atom {
                delegate: note.delegate,
                asset_id,
                amount: note.amount,
                nonce: note.nonce,
                signature: Some(signatures.len() as u32),
            });

            signatures.push(note.signature);
        }

        for (asset_id, amount) in self.outputs.into_iter() {
            atoms.push(Atom {
                delegate,
                asset_id,
                amount,
                nonce: Hash::random(&mut rand::rng()), // Generate random nonce
                signature: None,
            });
        }

        let refresh = Refresh {
            input_mask,
            atoms,
            asset_ids: self.assets.into_iter().collect(),
            signatures,
            blinded_points: vec![],
        };

        refresh.verify()?;

        Ok(refresh)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;
    use crate::types::*;

    #[proptest]
    fn prop_builder_produces_verifiable_refresh(
        policy_id: PolicyId,
        asset_name: AssetName,
        delegate: PublicKey,
        nonce: Hash,
        signature: Signature,
        #[strategy((0u32..=16).prop_map(|d| 1u64 << d))] amount: u64,
        #[strategy(proptest::collection::vec(1u64..=500_000, 1..=2))]
        split_weights: Vec<u64>,
    ) {
        let note = Note {
            amount,
            delegate,
            policy_id,
            asset_name,
            nonce,
            signature,
            dleq: None,
        };

        let total_weight: u64 = split_weights.iter().sum();
        let mut output_amounts: Vec<u64> = split_weights
            .iter()
            .map(|w| amount * w / total_weight)
            .collect();
        let output_sum: u64 = output_amounts.iter().sum();
        if output_sum < amount {
            output_amounts[0] += amount - output_sum;
        }

        let mut builder = RefreshBuilder::new().input(note);
        for &out_amount in &output_amounts {
            builder = builder.output(policy_id, asset_name, out_amount);
        }

        let refresh = builder.build().unwrap();
        prop_assert!(refresh.verify().is_ok());
        prop_assert!(refresh.atoms.iter().all(|a| a.amount.is_power_of_two()));
    }

    #[proptest]
    fn prop_builder_rejects_unbalanced(
        policy_id: PolicyId,
        asset_name: AssetName,
        delegate: PublicKey,
        nonce: Hash,
        signature: Signature,
        #[strategy((1u32..=16).prop_map(|d| 1u64 << d))] amount: u64,
    ) {
        let note = Note {
            amount,
            delegate,
            policy_id,
            asset_name,
            nonce,
            signature,
            dleq: None,
        };

        let builder = RefreshBuilder::new().input(note).output(
            policy_id,
            asset_name,
            amount + 1,
        );

        prop_assert!(builder.build().is_err());
    }

    /// Model-based: RefreshBuilder with multiple inputs on the same asset
    /// conserves per-asset total.
    ///
    /// Generates 2-4 input notes sharing one asset. The outputs split the
    /// aggregate total into a different number of outputs. Validates that
    /// the builder produces a verifiable Refresh where input_sum == output_sum.
    #[proptest]
    fn prop_builder_multi_input_conserves(
        policy_id: PolicyId,
        asset_name: AssetName,
        delegate: PublicKey,
        #[strategy(proptest::collection::vec(
            (0u32..=12).prop_map(|d| 1u64 << d),
            2..=4,
        ))]
        input_amounts: Vec<u64>,
        #[strategy(proptest::collection::vec(1u64..=100_000, 1..=2))]
        split_weights: Vec<u64>,
    ) {
        let total_input: u64 = input_amounts.iter().sum();

        let total_weight: u64 = split_weights.iter().sum();
        let mut output_amounts: Vec<u64> = split_weights
            .iter()
            .map(|w| total_input * w / total_weight)
            .collect();
        let output_sum: u64 = output_amounts.iter().sum();
        if output_sum < total_input {
            output_amounts[0] += total_input - output_sum;
        }

        let mut builder = RefreshBuilder::new();
        for (i, &amt) in input_amounts.iter().enumerate() {
            builder = builder.input(Note {
                amount: amt,
                delegate,
                policy_id,
                asset_name,
                nonce: Hash([i as u8; 32]),
                signature: Signature([i as u8; 32]),
                dleq: None,
            });
        }
        for &out_amount in &output_amounts {
            builder = builder.output(policy_id, asset_name, out_amount);
        }

        let output_atoms: usize =
            output_amounts.iter().map(|a| a.count_ones() as usize).sum();

        prop_assert_eq!(builder.input_count(), input_amounts.len());
        prop_assert_eq!(builder.output_count(), output_atoms);

        let refresh = builder.build().unwrap();
        prop_assert!(refresh.verify().is_ok());

        // Each output amount becomes one atom for each of its denominations.
        prop_assert_eq!(
            refresh.atoms.len(),
            input_amounts.len() + output_atoms
        );
    }
}
