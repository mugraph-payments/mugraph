use alloc::collections::BTreeMap;

use crate::types::*;

#[derive(Debug)]
pub struct TransactionBuilder {
    pub manifest: Manifest,
    pub cursor: usize,
    pub blob: Blob,
    pub asset_id_map: BTreeMap<Hash, u8>,
}

impl TransactionBuilder {
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            cursor: 0,
            blob: Blob::default(),
            asset_id_map: BTreeMap::default(),
        }
    }

    pub fn input(mut self, note: &Note) -> Self {
        let asset_id_index = match self.asset_id_map.get(&note.asset_id) {
            Some(&index) => index,
            None => {
                let index = self.asset_id_map.len() as u8;
                self.asset_id_map.insert(note.asset_id, index);
                self.blob.asset_ids[index as usize] = note.asset_id;
                index
            }
        };

        self.blob.asset_id_indexes[self.cursor] = asset_id_index;
        self.blob.amounts[self.cursor] = note.amount;
        self.blob.nonces[self.cursor] = note.nonce;
        self.blob.parent_ids[self.cursor] = note.parent_id;
        self.blob.program_ids[self.cursor] = note.program_id.unwrap_or_default();
        self.blob.data[self.cursor] = note.datum();

        self.cursor += 1;

        self
    }

    pub fn output(
        mut self,
        asset_id: Hash,
        amount: u64,
        program_id: Option<Hash>,
        datum: Option<Datum>,
    ) -> Self {
        let asset_id_index = match self.asset_id_map.get(&asset_id) {
            Some(&index) => index,
            None => {
                let index = self.asset_id_map.len() as u8;
                self.asset_id_map.insert(asset_id, index);
                self.blob.asset_ids[index as usize] = asset_id;
                index
            }
        };
        self.blob.asset_id_indexes[self.cursor] = asset_id_index;
        self.blob.amounts[self.cursor] = amount;
        self.blob.program_ids[self.cursor] = program_id.unwrap_or_default();
        self.blob.data[self.cursor] = datum.unwrap_or_default();

        self.cursor += 1;
        self
    }

    pub fn build(self) -> Transaction {
        Transaction {
            manifest: self.manifest,
            blob: self.blob,
        }
    }
}
