use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    types::{Atom, Hash, Note, PublicKey, Transaction},
};

mod coin_selection;

pub use coin_selection::*;

pub struct TransactionBuilder<S: CoinSelectionStrategy> {
    available_notes: Vec<Note>,
    outputs: Vec<(Hash, u64)>, // (asset_id, amount)
    coin_selection_strategy: S,
}

impl<S: CoinSelectionStrategy> TransactionBuilder<S> {
    pub fn new(strategy: S) -> Self {
        Self {
            available_notes: Vec::new(),
            outputs: Vec::new(),
            coin_selection_strategy: strategy,
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        self.available_notes.push(note);
        self
    }

    pub fn output(mut self, asset_id: Hash, amount: u64) -> Self {
        self.outputs.push((asset_id, amount));
        self
    }

    pub fn build(self) -> Result<Transaction> {
        let mut atoms = Vec::new();
        let mut asset_ids = Vec::new();
        let mut signatures = Vec::new();

        // Process inputs
        let mut input_map: HashMap<Hash, Vec<Note>> = HashMap::new();
        for note in self.available_notes {
            input_map.entry(note.asset_id).or_default().push(note);
        }

        // Process outputs
        for (asset_id, amount) in self.outputs {
            let selected_notes = self.coin_selection_strategy.select_coins(
                input_map.get(&asset_id).unwrap_or(&Vec::new()),
                amount,
                asset_id,
            );

            let total_input: u64 = selected_notes.iter().map(|n| n.amount).sum();
            if total_input < amount {
                return Err(Error::InsufficientFunds {
                    asset_id,
                    expected: amount,
                    got: total_input,
                });
            }

            // Add input atoms
            for note in selected_notes.iter() {
                atoms.push(Atom {
                    delegate: note.delegate,
                    asset_id: asset_ids.len() as u32,
                    amount: note.amount,
                    nonce: note.nonce,
                    signature: Some(signatures.len() as u32),
                });
                signatures.push(note.signature);
            }

            // Add output atom
            atoms.push(Atom {
                delegate: PublicKey::zero(), // This should be set by the recipient
                asset_id: asset_ids.len() as u32,
                amount,
                nonce: Hash::zero(), // This should be set by the recipient
                signature: None,
            });

            // Add change atom if necessary
            if total_input > amount {
                atoms.push(Atom {
                    delegate: selected_notes[0].delegate, // Use the first note's delegate for change
                    asset_id: asset_ids.len() as u32,
                    amount: total_input - amount,
                    nonce: Hash::zero(), // This should be set by the wallet
                    signature: None,
                });
            }

            asset_ids.push(asset_id);
        }

        Ok(Transaction {
            atoms,
            asset_ids,
            signatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::{collection::vec, prelude::*};
    use test_strategy::proptest;

    use super::*;
    use crate::types::Signature;

    // Helper function to create a Note
    fn create_note(asset_id: Hash, amount: u64, delegate: PublicKey) -> Note {
        Note {
            amount,
            delegate,
            asset_id,
            nonce: Hash::default(),
            signature: Signature::default(),
        }
    }

    #[proptest]
    fn test_build_simple(asset_id: Hash, #[strategy(1..1000u64)] amount: u64, delegate: PublicKey) {
        let input = create_note(asset_id, amount, delegate);
        let tx = TransactionBuilder::new(GreedyCoinSelection)
            .input(input)
            .output(asset_id, amount)
            .build()?;

        prop_assert_eq!(tx.asset_ids.len(), 1);
        prop_assert_eq!(tx.asset_ids[0], asset_id);

        let inputs: Vec<_> = tx.atoms.iter().filter(|a| a.signature.is_some()).collect();
        let outputs: Vec<_> = tx.atoms.iter().filter(|a| a.signature.is_none()).collect();

        prop_assert_eq!(inputs.len(), 1);
        prop_assert_eq!(outputs.len(), 1);
        prop_assert_eq!(inputs[0].amount, amount);
        prop_assert_eq!(outputs[0].amount, amount);
    }

    #[proptest]
    fn test_insufficient_funds(
        asset_id: Hash,
        #[strategy(1..1000u64)] amount: u64,
        delegate: PublicKey,
    ) {
        let input = create_note(asset_id, amount, delegate);
        let result = TransactionBuilder::new(GreedyCoinSelection)
            .input(input)
            .output(asset_id, amount + 1)
            .build();

        prop_assert!(result.is_err());
    }

    #[proptest]
    fn test_multiple_inputs_and_outputs(
        asset_id1: Hash,
        asset_id2: Hash,
        #[strategy(1..500u64)] amount1: u64,
        #[strategy(1..500u64)] amount2: u64,
        #[strategy(1..=#amount1)] output1: u64,
        #[strategy(1..=#amount2)] output2: u64,
        delegate: PublicKey,
    ) {
        prop_assume!(asset_id1 != asset_id2);

        let input1 = create_note(asset_id1, amount1, delegate);
        let input2 = create_note(asset_id2, amount2, delegate);

        let tx = TransactionBuilder::new(GreedyCoinSelection)
            .input(input1)
            .input(input2)
            .output(asset_id1, output1)
            .output(asset_id2, output2)
            .build()?;

        prop_assert_eq!(tx.asset_ids.len(), 2);
        prop_assert!(tx.asset_ids.contains(&asset_id1));
        prop_assert!(tx.asset_ids.contains(&asset_id2));

        let mut input_totals = HashMap::new();
        let mut output_totals = HashMap::new();

        for atom in &tx.atoms {
            let asset_id = tx.asset_ids[atom.asset_id as usize];
            if atom.signature.is_some() {
                *input_totals.entry(asset_id).or_insert(0) += atom.amount;
            } else {
                *output_totals.entry(asset_id).or_insert(0) += atom.amount;
            }
        }

        prop_assert_eq!(input_totals.len(), 2);
        prop_assert_eq!(output_totals.len(), 2);
        prop_assert_eq!(*input_totals.get(&asset_id1).unwrap(), amount1);
        prop_assert_eq!(*input_totals.get(&asset_id2).unwrap(), amount2);
        prop_assert_eq!(*output_totals.get(&asset_id1).unwrap(), amount1);
        prop_assert_eq!(*output_totals.get(&asset_id2).unwrap(), amount2);
    }

    #[proptest]
    fn test_change_generation(
        asset_id: Hash,
        #[strategy(100..1000u64)] input_amount: u64,
        #[strategy(1..100u64)] output_amount: u64,
        delegate: PublicKey,
    ) {
        let input = create_note(asset_id, input_amount, delegate);
        let tx = TransactionBuilder::new(GreedyCoinSelection)
            .input(input)
            .output(asset_id, output_amount)
            .build()?;

        let inputs: Vec<_> = tx.atoms.iter().filter(|a| a.signature.is_some()).collect();
        let outputs: Vec<_> = tx.atoms.iter().filter(|a| a.signature.is_none()).collect();

        prop_assert_eq!(inputs.len(), 1);
        prop_assert_eq!(outputs.len(), 2); // One for output, one for change

        let change_output = outputs
            .iter()
            .find(|a| a.amount == input_amount - output_amount);
        prop_assert!(change_output.is_some());
    }

    #[proptest]
    fn test_multiple_asset_types(
        delegate: PublicKey,
        #[strategy(vec(any::<Hash>(), 1..100))] asset_ids: Vec<Hash>,
        #[strategy(vec(1..u64::MAX, #asset_ids.len()..=#asset_ids.len()))] amounts: Vec<u64>,
    ) {
        let mut builder = TransactionBuilder::new(GreedyCoinSelection);

        for (&asset_id, &amount) in asset_ids.iter().zip(amounts.iter()) {
            let input = create_note(asset_id, amount, delegate);
            builder = builder.input(input).output(asset_id, amount / 2);
        }

        let tx = builder.build()?;

        prop_assert_eq!(tx.asset_ids.len(), asset_ids.len());

        for &asset_id in &asset_ids {
            let asset_inputs: Vec<_> = tx
                .atoms
                .iter()
                .filter(|a| tx.asset_ids[a.asset_id as usize] == asset_id && a.signature.is_some())
                .collect();
            let asset_outputs: Vec<_> = tx
                .atoms
                .iter()
                .filter(|a| tx.asset_ids[a.asset_id as usize] == asset_id && a.signature.is_none())
                .collect();

            prop_assert_eq!(asset_inputs.len(), 1);
            prop_assert_eq!(asset_outputs.len(), 2); // One for output, one for change
        }
    }
}
