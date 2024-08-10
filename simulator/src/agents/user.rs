use std::collections::HashMap;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use rand::{CryptoRng, RngCore};

use super::Agent;
use crate::util::Location;

pub struct User {
    pub location: Location,
    pub notes: Vec<Note>,
    pub balances: HashMap<Hash, u64>,
}

impl User {
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self {
            location: Location::random(&mut rng),
            notes: vec![],
            balances: HashMap::new(),
        }
    }
}

#[async_trait]
impl Agent for User {
    type Input = Note;
    type Output = ();

    async fn recv(&mut self, note: Self::Input) -> Result<Self::Output> {
        let amount = note.amount;

        self.balances
            .entry(note.asset_id)
            .and_modify(|x| *x += amount)
            .or_insert(amount);
        self.notes.push(note);

        Ok(())
    }
}
