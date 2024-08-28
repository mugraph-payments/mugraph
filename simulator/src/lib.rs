#![feature(duration_millis_float)]

use std::time::Instant;

use agents::user::User;
use color_eyre::eyre::Result;
use metrics::{counter, histogram};
use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    types::*,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use self::agents::delegate::Delegate;
pub use self::config::Config;

mod agents;
mod config;

pub enum Action {
    Transfer {
        from: u32,
        to: u32,
        asset_id: Hash,
        amount: u64,
    },
}

impl Action {
    pub fn random(rng: &mut ChaCha20Rng, config: &Config, sim: &Simulation) -> Self {
        // We only have one type (for now)
        match 0 {
            0 => {
                let from = rng.gen_range(0..config.users) as u32;
                let to = rng.gen_range(0..config.users) as u32;

                let note = sim.users[from as usize].notes.choose(rng).unwrap();
                let asset_id = note.asset_id;
                let amount = rng.gen_range(1..note.amount);

                Self::Transfer {
                    from,
                    to,
                    asset_id,
                    amount,
                }
            }
            _ => unreachable!(),
        }
    }
}

pub struct Simulation {
    rng: ChaCha20Rng,
    config: Config,
    users: Vec<User>,
    delegate: Delegate,
}

impl Simulation {
    pub fn new(config: Config) -> Result<Self> {
        let mut rng = config.rng();
        let mut delegate = Delegate::new(&config);
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut users = vec![];

        for _ in 0..config.users {
            let mut notes = vec![];

            for _ in 0..rng.gen_range(1..config.notes) {
                let idx = rng.gen_range(0..config.assets);

                let asset_id = assets[idx];
                let amount = rng.gen_range(1..1_000_000_000);

                let note = delegate.emit(asset_id, amount)?;

                notes.push(note);
            }

            assert_ne!(notes.len(), 0);
            let mut user = User::new();
            user.notes = notes;
            users.push(user);
        }

        Ok(Self {
            rng,
            users,
            delegate,
            config,
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        let mut rng = self.rng.clone();

        match Action::random(&mut rng, &self.config, &self) {
            Action::Transfer {
                from,
                to,
                asset_id,
                amount,
            } => {
                let start = Instant::now();
                let note_idx = self.users[from as usize]
                    .notes
                    .iter()
                    .position(|note| note.asset_id == asset_id && note.amount >= amount)
                    .expect("No suitable note found");
                let note = self.users[from as usize].notes.remove(note_idx);

                let mut transaction = TransactionBuilder::new(GreedyCoinSelection)
                    .input(note.clone())
                    .output(note.asset_id, amount);

                if amount < note.amount {
                    transaction = transaction.output(note.asset_id, note.amount - amount);
                }

                let response = self.delegate.recv_transaction_v0(transaction.build()?)?;

                match response {
                    V0Response::Transaction { outputs } => {
                        // Create new notes from the outputs
                        for (i, blinded_sig) in outputs.iter().enumerate() {
                            let new_note = Note {
                                amount: if i == 0 { amount } else { note.amount - amount },
                                delegate: note.delegate,
                                asset_id: note.asset_id,
                                nonce: Hash::random(&mut self.rng),
                                signature: crypto::unblind_signature(
                                    blinded_sig,
                                    &crypto::blind(&mut self.rng, &[]).factor,
                                    &self.delegate.public_key(),
                                )
                                .expect("Failed to unblind signature"),
                            };

                            if i == 0 {
                                self.users[to as usize].notes.push(new_note);
                            } else {
                                // Keep the change
                                self.users[from as usize].notes.push(new_note);
                            }
                        }
                    }
                }

                counter!("mugraph.simulator.ticks").increment(1);
                counter!("mugraph.simulator.processed_transactions").increment(1);
                histogram!("mugraph.simulator.time_taken").record(start.elapsed().as_millis_f64());
            }
        }

        Ok(())
    }
}
