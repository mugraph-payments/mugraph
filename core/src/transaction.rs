use serde::{Deserialize, Serialize};

use crate::Note;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub presence: u8,
    pub kinds: u8,
    pub asset_ids: [[u8; 32]; 8],
    pub amounts: [(u8, u64); 8],
    pub nullifiers: [[u8; 32]; 8],
}

pub struct TransactionBuilder<'a> {
    dst: &'a mut Transaction,
}

impl<'a> TransactionBuilder<'a> {
    pub fn new(dst: &'a mut Transaction) -> Self {
        TransactionBuilder { dst }
    }

    pub fn add_input(self, note: Note) -> Result<Self, &'static str> {
        if self.dst.presence == 0xFF {
            return Err("Transaction is full");
        }

        self.add_note(note, false)
    }

    pub fn add_output(self, note: Note) -> Result<Self, &'static str> {
        if self.dst.presence == 0xFF {
            return Err("Transaction is full");
        }

        self.add_note(note, true)
    }

    fn add_note(self, note: Note, is_output: bool) -> Result<Self, &'static str> {
        let mut index = 0;
        for i in 0..8 {
            if self.dst.presence & (1 << i) == 0 {
                index = i;
                break;
            }
        }

        let mut asset_id_position = 8;
        for i in 0..8 {
            if self.dst.asset_ids[i] == note.asset_id {
                asset_id_position = i;
                break;
            }
        }

        if asset_id_position == 8 {
            for i in 0..8 {
                if self.dst.asset_ids[i] == [0u8; 32] {
                    self.dst.asset_ids[i] = note.asset_id;
                    asset_id_position = i;
                    break;
                }
            }
        }

        self.dst.amounts[index] = (asset_id_position as u8, note.amount);
        self.dst.nullifiers[index] = note.nullifier;
        self.dst.presence |= 1 << index;
        if is_output {
            self.dst.kinds |= 1 << index;
        }

        Ok(self)
    }

    pub fn build(self) -> Result<Self, &'static str> {
        let input_count = (0..8)
            .filter(|&i| (self.dst.presence & (1 << i) != 0) && (self.dst.kinds & (1 << i) == 0))
            .count();
        let output_count = (0..8)
            .filter(|&i| (self.dst.presence & (1 << i) != 0) && (self.dst.kinds & (1 << i) != 0))
            .count();

        if input_count == 0 || output_count == 0 {
            return Err("Transaction must have at least one input and one output");
        }

        Ok(self)
    }
}
