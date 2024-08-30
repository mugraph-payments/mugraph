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
use tracing::{error, warn};

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
    pub fn new(config: Config) -> Result<Self> {
        let mut rng = config.rng();
        let mut delegate = Self::build_delegate(&config)?;
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

    fn build_delegate(config: &Config) -> Result<Delegate, Error> {
        loop {
            match Delegate::new(config) {
                Err(Error::StorageError { reason: _ }) => continue,
                v => return Ok(v?),
            }
        }
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
        let mut rng = self.rng.clone();

        match Action::random(&mut rng, &self.config, self) {
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

                loop {
                    match self.process_transaction(transaction.clone(), &note, from, to, amount) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Transaction failed, retrying: {e}");
                            match self.process_transaction(
                                transaction.clone(),
                                &note,
                                from,
                                to,
                                amount,
                            ) {
                                Err(e @ Error::AlreadySpent { signature }) => {
                                    error!("Consistency error: {signature} was spent on a failed transaction");
                                    Err(e)?;
                                }
                                Ok(_) => break,
                                Err(_) => continue,
                            }
                        }
                    }
                }

                histogram!("mugraph.simulator.time_elapsed")
                    .record(start.elapsed().as_millis_f64());
            }
        }

        Ok(())
    }
}
