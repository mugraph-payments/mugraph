#![feature(duration_millis_float)]
#![feature(integer_atomics)]

use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::{error::Error, inc, types::*, utils::timed};
use rand::prelude::*;
use tracing::{debug, warn};

mod action;
mod config;
mod delegate;
pub mod observer;
mod state;
mod tick;

pub use self::{action::Action, config::Config, delegate::Delegate, state::State, tick::tick};

pub struct Simulation {
    core_id: u32,
    state: State,
    delegate: Delegate,
}

impl Simulation {
    pub fn new<R: CryptoRng + Rng>(
        rng: &mut R,
        core_id: u32,
        mut delegate: Delegate,
    ) -> Result<Self, Error> {
        Ok(Self {
            core_id,
            state: State::setup(rng, &mut delegate)?,
            delegate,
        })
    }

    #[tracing::instrument(skip(self))]
    #[timed]
    pub fn tick(&mut self, round: u64) -> Result<(), Error> {
        debug!(
            core_id = self.core_id,
            round = round,
            "Starting simulation tick"
        );

        let action = self.state.next_action()?;

        loop {
            match self.handle_action(&action) {
                Ok(_) => break,
                Err(Error::SimulatedError { reason }) => {
                    counter!("mugraph.resources", "name" => "retries", "reason" => reason)
                        .increment(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    #[timed]
    fn handle_action(&mut self, action: &Action) -> Result<(), Error> {
        match action {
            Action::Transaction(transaction) => {
                let response = self.delegate.recv_transaction_v0(transaction)?;

                match response {
                    V0Response::Transaction { outputs } => {
                        let mut index = 0;

                        for (i, atom) in transaction.atoms.iter().enumerate() {
                            if transaction.is_input(i) {
                                continue;
                            }

                            let asset_id = transaction.asset_ids[atom.asset_id as usize];

                            self.state.recv(asset_id, atom.amount, outputs[index])?;

                            index += 1;
                        }

                        counter!("transactions").increment(1);
                    }
                }
            }
            Action::DoubleSpend(transaction) => {
                self.delegate.recv_transaction_v0(transaction)?;

                match self.delegate.recv_transaction_v0(transaction) {
                    Ok(_) => {
                        return Err(Error::SimulationError {
                            reason: "Expected redemption to block double spend".to_string(),
                        })
                    }
                    Err(Error::AlreadySpent { .. }) => {
                        inc!("blocked_double_spent");
                    }
                    Err(e) => return Err(e),
                }

                inc!("double_spends");
            }
        }

        Ok(())
    }
}
