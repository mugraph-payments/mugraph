#![feature(duration_millis_float)]

use std::time::Instant;

use agents::user::User;
use color_eyre::eyre::Result;
use metrics::{counter, histogram};
use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    error::Error,
    types::*,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::{error, info};

mod action;
mod agents;
mod config;

pub use self::{action::Action, agents::delegate::Delegate, config::Config};

pub struct Simulation {
    rng: ChaCha20Rng,
    config: Config,
    users: Vec<User>,
    delegate: Delegate,
}

impl Simulation {
    #[tracing::instrument(skip(config, delegate))]
    pub fn new(core_id: u32, config: Config, mut delegate: Delegate) -> Result<Self> {
        let mut rng = config.rng();

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

        info!("Simulation initialized");

        Ok(Self {
            rng,
            users,
            delegate,
            config,
        })
    }

    pub fn process_transaction(
        &mut self,
        transaction: Transaction,
        note: &Note,
        from: u32,
        to: u32,
        amount: u64,
    ) -> Result<(), Error> {
        let response = self.delegate.recv_transaction_v0(transaction)?;

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
                        )?,
                    };

                    if i == 0 {
                        self.users[to as usize].notes.push(new_note);
                    } else {
                        // Keep the change
                        self.users[from as usize].notes.push(new_note);
                    }
                }
            }
            V0Response::Error { errors } => {
                return Err(errors[0].clone())?;
            }
        }

        counter!("mugraph.simulator.processed_transactions").increment(1);

        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        let action = Action::random(&self.config.clone(), self);

        match action {
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
                    .ok_or(Error::InsufficientFunds {
                        asset_id,
                        expected: amount,
                        got: 0,
                    })?;
                let note = self.users[from as usize].notes.remove(note_idx);

                let mut transaction = TransactionBuilder::new(GreedyCoinSelection)
                    .input(note.clone())
                    .output(note.asset_id, amount);

                if amount < note.amount {
                    transaction = transaction.output(note.asset_id, note.amount - amount);
                }

                let transaction = transaction.build()?;

                match self.process_transaction(transaction.clone(), &note, from, to, amount) {
                    Ok(_) => {}
                    Err(Error::StorageError { .. }) => {
                        counter!("mugraph.simulator.injected_failures").increment(1);

                        match self.process_transaction(transaction.clone(), &note, from, to, amount)
                        {
                            Err(e @ Error::AlreadySpent { signature }) => {
                                error!("Consistency error: {signature} was spent on a failed transaction");
                                Err(e)?;
                            }
                            Err(e @ Error::StorageError { reason: _ }) => {
                                counter!("mugraph.simulator.injected_failures").increment(1);
                                Err(e)?;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                Err(e)?;
                            }
                        }
                    }
                    Err(e) => {
                        Err(e)?;
                    }
                }

                histogram!("mugraph.simulator.time_elapsed")
                    .record(start.elapsed().as_millis_f64());
            }
        }

        Ok(())
    }
}
