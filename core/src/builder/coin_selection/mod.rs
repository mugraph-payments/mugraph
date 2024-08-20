mod greedy;

pub use self::greedy::*;
use crate::types::*;

pub trait CoinSelectionStrategy {
    fn select_coins(&self, available: &[Note], target: u64, asset_id: Hash) -> Vec<Note>;
}
