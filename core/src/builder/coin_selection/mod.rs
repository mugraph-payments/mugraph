mod greedy;
mod knapsack;

pub use self::{greedy::*, knapsack::*};
use crate::types::*;

pub trait CoinSelectionStrategy {
    fn select_coins(&self, available: &[Note], target: u64, asset_id: Hash) -> Vec<Note>;
}
