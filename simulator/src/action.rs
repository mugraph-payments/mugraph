use mugraph_core::types::*;
use rand::prelude::*;

use crate::{Config, Simulation};

pub enum Action {
    Transfer {
        from: u32,
        to: u32,
        asset_id: Hash,
        amount: u64,
    },
}

impl Action {
    pub fn random(config: &Config, sim: &mut Simulation) -> Self {
        // We only have one type (for now)
        match 0 {
            0 => loop {
                let from = sim.rng.gen_range(0..config.users) as u32;

                if sim.users[from as usize].notes.is_empty() {
                    continue;
                }

                let to = sim.rng.gen_range(0..config.users) as u32;

                let note = sim.users[from as usize].notes.choose(&mut sim.rng).unwrap();
                let asset_id = note.asset_id;

                if note.amount == 1 {
                    continue;
                }

                let amount = sim.rng.gen_range(1..note.amount);

                return Self::Transfer {
                    from,
                    to,
                    asset_id,
                    amount,
                };
            },
            _ => unreachable!(),
        }
    }
}
