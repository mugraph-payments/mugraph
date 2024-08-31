#![feature(duration_millis_float)]

use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use color_eyre::eyre::Result;
use metrics::{counter, histogram};
use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    error::Error,
    types::*,
};
use rand_chacha::ChaCha20Rng;
use state::State;
use tracing::{error, info};

mod action;
mod agents;
mod config;
mod state;

pub use self::{
    action::Action,
    agents::{delegate::Delegate, user::User},
    config::Config,
};

pub struct Simulation {
    rng: ChaCha20Rng,
    delegate: Delegate,
    state: State,
    users: Arc<RwLock<Vec<User>>>,
}

impl Simulation {
    pub fn new(
        core_id: u32,
        config: Config,
        delegate: Delegate,
        users: Arc<RwLock<Vec<User>>>,
    ) -> Result<Self> {
        let mut rng = config.rng();

        info!(simulation_id = core_id, "Simulation initialized");

        Ok(Self {
            users,
            delegate,
            state: State::new(&mut rng),
            rng,
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
        let mut users = self.users.write()?;

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
                        users[to as usize].notes.push(new_note);
                    } else {
                        // Keep the change
                        users[from as usize].notes.push(new_note);
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

    pub fn tick(&mut self) -> Result<(), Error> {
        let action = self.state.next(self.users.clone())?;
        let u = self.users.clone();
        let mut users = u.write()?;

        match action {
            Action::Transfer {
                from,
                to,
                asset_id,
                amount,
            } => {
                let start = Instant::now();
                let note_idx = users[from as usize]
                    .notes
                    .iter()
                    .position(|note| note.asset_id == asset_id && note.amount >= amount)
                    .ok_or(Error::InsufficientFunds {
                        asset_id,
                        expected: amount,
                        got: 0,
                    })?;
                let note = users[from as usize].notes.remove(note_idx);

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
