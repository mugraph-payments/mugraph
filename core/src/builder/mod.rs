use indexmap::{IndexMap, IndexSet};

use crate::{
    error::{Error, Result},
    timed,
    types::{Atom, Hash, Note, Transaction},
    utils::BitSet32,
};

#[derive(Default)]
pub struct TransactionBuilder {
    inputs: Vec<Note>,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
    assets: IndexSet<Hash>,
    outputs: Vec<(u32, u64)>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            pre_balances: vec![0; 8],
            post_balances: vec![0; 8],
            ..Default::default()
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        match self.assets.get_index_of(&note.asset_id) {
            Some(i) => {
                self.pre_balances[i] += note.amount;
            }
            None => {
                self.assets.insert(note.asset_id);
            }
        }

        self.inputs.push(note);

        self
    }

    pub fn output(mut self, asset_id: Hash, amount: u64) -> Self {
        match self.assets.get_index_of(&asset_id) {
            Some(i) => {
                self.post_balances[i] += amount;
                self.outputs.push((i as u32, amount));
            }
            None => {
                self.post_balances[self.assets.len()] += amount;
                self.assets.insert(asset_id);
            }
        }

        self
    }

    pub fn build(self) -> Result<Transaction> {
        let mut atoms = Vec::new();
        let mut signatures = Vec::new();
        let mut input_mask = BitSet32::new();
        let delegate = self.inputs[0].delegate;

        // Process inputs
        for (index, note) in self.inputs.into_iter().enumerate() {
            input_mask.insert(index as u32);

            let asset_id = match self.assets.get_index_of(&note.asset_id) {
                Some(a) => a as u32,
                None => {
                    return Err(Error::InvalidTransaction {
                        reason: "Missing asset_id for iput".to_string(),
                    })
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

        // Process outputs
        for (asset_id, amount) in self.outputs.into_iter() {
            atoms.push(Atom {
                delegate, // Assuming all inputs have the same delegate
                asset_id,
                amount,
                nonce: Hash::zero(), // TODO: Generate a nonce for outputs
                signature: None,
            });
        }

        let transaction = Transaction {
            input_mask,
            atoms,
            asset_ids: self.assets.into_iter().collect(),
            signatures,
        };

        transaction.verify()?;

        Ok(transaction)
    }
}
