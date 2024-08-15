use std::collections::HashMap;

use mugraph_client::prelude::*;

#[derive(Default)]
pub struct User {
    pub notes: Vec<Note>,
    pub balances: HashMap<Hash, u64>,
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn balance(&self, asset_id: Hash) -> u64 {
        *self.balances.get(&asset_id).unwrap_or(&0)
    }
}
