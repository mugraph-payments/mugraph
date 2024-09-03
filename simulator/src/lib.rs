#![feature(duration_millis_float)]

use std::time::Instant;

use color_eyre::eyre::Result;
use metrics::{counter, histogram};
use mugraph_core::{error::Error, timed, types::*};
use tracing::debug;

mod action;
mod config;
mod delegate;
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

    pub fn tick(&mut self, round: u64) -> Result<(), Error> {
        let start = Instant::now();

        debug!(
            core_id = self.core_id,
            round = round,
            "Starting simulation tick"
        );

        let action = timed!("mugraph.simulator.state.next.time_taken", {
            self.state.next_action()?
        });

        match action {
            Action::Split(transaction) | Action::Join(transaction) => {
                let response = self.delegate.recv_transaction_v0(&transaction)?;

                match response {
                    V0Response::Transaction { outputs } => {
                        let mut index = 0;

                        for (i, atom) in transaction.atoms.iter().enumerate() {
                            counter!("mugraph.simulator.atoms_processed").increment(1);

                            if transaction.is_input(i) {
                                counter!("mugraph.simulator.inputs_processed").increment(1);

                                continue;
                            }

                            let asset_id = transaction.asset_ids[atom.asset_id as usize];

                            self.state.recv(asset_id, atom.amount, outputs[index])?;

                            counter!("mugraph.simulator.outputs_received").increment(1);

                            index += 1;
                        }

                        counter!("mugraph.simulator.transactions_processed").increment(1);
                    }
                }
            }
        }

        histogram!("mugraph.simulator.tick.time_taken").record(start.elapsed().as_millis_f64());

        Ok(())
    }
}
