use std::collections::BTreeMap;

use crate::prelude::{Hash, Note, Transaction};

#[derive(Debug, Default)]
pub struct TransactionBuilder {
    pub cursor: u8,
    pub transaction: Transaction,
    pub asset_id_map: BTreeMap<Hash, u8>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn input(mut self, note: &Note) -> Self {
        let asset_id_index = match self.asset_id_map.get(&note.asset_id) {
            Some(&index) => index,
            None => {
                let index = self.asset_id_map.len() as u8;
                self.asset_id_map.insert(note.asset_id, index);
                self.transaction.asset_ids[index as usize] = note.asset_id;
                index
            }
        };

        self.transaction.input_mask.insert(self.cursor);
        self.transaction.asset_id_indexes[self.cursor as usize] = asset_id_index;
        self.transaction.amounts[self.cursor as usize] = note.amount;
        self.transaction.commitments[self.cursor as usize] = note.nonce;

        self.cursor += 1;

        self
    }

    pub fn output(mut self, asset_id: Hash, amount: u64) -> Self {
        let asset_id_index = match self.asset_id_map.get(&asset_id) {
            Some(&index) => index,
            None => {
                let index = self.asset_id_map.len() as u8;
                self.asset_id_map.insert(asset_id, index);
                self.transaction.asset_ids[index as usize] = asset_id;
                index
            }
        };
        self.transaction.asset_id_indexes[self.cursor as usize] = asset_id_index;
        self.transaction.amounts[self.cursor as usize] = amount;

        self.cursor += 1;
        self
    }

    pub fn build(self) -> Transaction {
        self.transaction
    }
}
