use std::collections::HashMap;

use mugraph_client::prelude::*;

#[derive(Default, PartialEq, Eq)]
pub struct User {
    pub id: usize,
    pub notes: Vec<Note>,
    pub balances: HashMap<Hash, u64>,
}

impl User {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    pub fn balance(&self, asset_id: Hash) -> u64 {
        *self.balances.get(&asset_id).unwrap_or(&0)
    }
}
