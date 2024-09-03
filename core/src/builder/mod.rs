use crate::{
    error::Result,
    types::{Atom, Hash, Note, Transaction},
};

pub struct TransactionBuilder {
    available_notes: Vec<Note>,
    outputs: Vec<(Hash, u64)>, // (asset_id, amount)
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            available_notes: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        self.available_notes.push(note);
        self
    }

    pub fn input_count(&self) -> usize {
        self.available_notes.len()
    }

    pub fn output(mut self, asset_id: Hash, amount: u64) -> Self {
        self.outputs.push((asset_id, amount));
        self
    }

    pub fn build(self) -> Result<Transaction> {
        let mut atoms = Vec::new();
        let mut asset_ids = Vec::new();
        let mut signatures = Vec::new();

        assert_ne!(self.input_count(), 0);

        let delegate = self.available_notes[0].delegate;

        for note in self.available_notes {
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

        Ok(Transaction {
            atoms,
            asset_ids,
            signatures,
        })
    }
}
