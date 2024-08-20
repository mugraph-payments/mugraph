use crate::{builder::CoinSelectionStrategy, types::*};

pub struct GreedyCoinSelection;

impl CoinSelectionStrategy for GreedyCoinSelection {
    fn select_coins(&self, available: &[Note], target: u64, asset_id: Hash) -> Vec<Note> {
        let mut selected = Vec::new();
        let mut total = 0;

        for note in available {
            if note.asset_id == asset_id {
                selected.push(note.clone());
                total += note.amount;
                if total >= target {
                    break;
                }
            }
        }

        selected
    }
}
