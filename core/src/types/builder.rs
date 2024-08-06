use crate::types::*;

#[derive(Debug)]
pub struct TransactionBuilder {
    pub manifest: Manifest,
    pub cursor: usize,
    pub blob: Blob,
}

impl TransactionBuilder {
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            cursor: 0,
            blob: Blob::default(),
        }
    }

    pub fn input(mut self, note: &Note) -> Self {
        self.blob.asset_ids[self.cursor] = note.asset_id;
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
        self.blob.asset_ids[self.cursor] = asset_id;
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
