use std::sync::{Arc, RwLock};

use metrics::counter;
use mugraph_core::error::Error;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::{agents::user::User, Action};

pub struct State {
    rng: ChaCha20Rng,
}

impl State {
    pub fn new<R: CryptoRng + Rng>(rng: &mut R) -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
        }
    }

    pub fn next(&mut self, users: Arc<RwLock<Vec<User>>>) -> Result<Action, Error> {
        let kind = self.rng.gen_range(0..=0);
        let users = users.read()?;

        match kind {
            0 => {
                let mut from = None;

                while from.is_none() {
                    let id = self.rng.gen_range(0..users.len());

                    if !users[id].notes.is_empty() {
                        counter!("mugraph.simulator.state.transfer.skipped_sender").increment(1);
                        from = Some(id);
                    }
                }

                let from = from.unwrap();
                let to = self.rng.gen_range(0..users.len());

                let note = users[from].notes.choose(&mut self.rng).unwrap();
                let asset_id = note.asset_id;

                if note.amount == 1 {
                    panic!();
                }

                let amount = self.rng.gen_range(1..note.amount);

                Ok(Action::Transfer {
                    from: from as u32,
                    to: to as u32,
                    asset_id,
                    amount,
                })
            }
            _ => unreachable!(),
        }
    }
}
