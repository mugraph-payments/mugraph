#![feature(duration_millis_float)]

use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::{error::Error, inc, timed, types::*};
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
    pub fn new(core_id: u32, delegate: Delegate) -> Result<Self, Error> {
        Ok(Self {
            core_id,
            state: State::setup(delegate.clone())?,
            delegate,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn tick(&mut self, round: u64) -> Result<(), Error> {
        debug!(
            core_id = self.core_id,
            round = round,
            "Starting simulation tick"
        );

        let action = timed!("state.next", { self.state.next_action(round)? });

        loop {
            match timed!("handle_action", { self.handle_action(round, &action) }) {
                Ok(_) => break,
                Err(Error::SimulatedError { reason }) => {
                    counter!("mugraph.resources", "name" => "user_retries", "reason" => reason)
                        .increment(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn handle_action(&mut self, round: u64, action: &Action) -> Result<(), Error> {
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

                        inc!("transactions");
                    }
                }
            }
            Action::RedeemFail(transaction) => {
                match self.handle_action(round, &Action::Transaction(transaction.clone())) {
                    Ok(_) => {
                        return Err(Error::SimulationError {
                            reason: "Expected redemption to block double spend, but it didn't"
                                .to_string(),
                        })
                    }
                    Err(Error::AlreadySpent { .. }) => {
                        inc!("blocked_double_spent");
                    }
                    e => e?,
                }
            }
            Action::DoubleSpend(a) => {
                self.state
                    .schedule(round + 1, Action::Transaction(a.clone()));
                self.state
                    .schedule(round + 2, Action::RedeemFail(a.clone()));

                inc!("double_spends");
            }
        }

        Ok(())
    }
}
