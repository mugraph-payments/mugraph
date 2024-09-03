use crate::{
    error::Result,
    types::{Atom, Hash, Note, Transaction},
    utils::BitSet32,
};

pub struct TransactionBuilder {
    inputs: Vec<Note>,
    outputs: Vec<(Hash, u64)>, // (asset_id, amount)
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBuilder {
    pub const fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        self.inputs.push(note);
        self
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn output(mut self, asset_id: Hash, amount: u64) -> Self {
        self.outputs.push((asset_id, amount));
        self
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn build(self) -> Result<Transaction> {
        let mut atoms = Vec::with_capacity(self.input_count() + self.output_count());
        let mut asset_ids = Vec::new();
        let mut signatures = Vec::with_capacity(self.input_count());
        let mut input_mask = BitSet32::new();

        assert_ne!(self.input_count(), 0);

        let delegate = self.inputs[0].delegate;

        for (i, note) in self.inputs.iter().enumerate() {
            input_mask.insert(i as u32);

            atoms.push(Atom {
                delegate: note.delegate,
                asset_id: asset_ids.len() as u32,
                amount: note.amount,
                nonce: note.nonce,
                signature: Some(signatures.len() as u32),
            });

            signatures.push(note.signature);

            if !asset_ids.contains(&note.asset_id) {
                asset_ids.push(note.asset_id);
            }
        }

        for (asset_id, amount) in self.outputs {
            atoms.push(Atom {
                delegate,
                asset_id: asset_ids.iter().position(|&id| id == asset_id).unwrap() as u32,
                amount,
                nonce: Hash::zero(),
                signature: None,
            });
            if !asset_ids.contains(&asset_id) {
                asset_ids.push(asset_id);
            }
        }

        let transaction = Transaction {
            input_mask,
            atoms,
            asset_ids,
            signatures,
        };

        transaction.verify()?;

        Ok(transaction)
    }
}
