#![feature(duration_millis_float)]

use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::{error::Error, timed, types::*};
use tracing::{debug, warn};

mod action;
mod config;
mod delegate;
pub mod observer;
mod state;

pub use self::{action::Action, config::Config, delegate::Delegate, state::State};

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

        let action = timed!("state.next", { self.state.next_action()? });

        loop {
            match timed!("handle_action", { self.handle_action(&action) }) {
                Ok(_) => break,
                Err(Error::SimulatedError { reason }) => {
                    counter!("user_retries", "reason" => reason).increment(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn handle_action(&mut self, action: &Action) -> Result<(), Error> {
        match action {
            Action::Split(transaction) | Action::Join(transaction) => {
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

                            counter!("outputs_received").increment(1);

                            index += 1;
                        }

                        counter!("transactions_processed").increment(1);
                    }
                }
            }
        }

        Ok(())
    }
}
