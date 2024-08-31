use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::{error::Error, types::*};
use tracing::info;

mod action;
mod config;
mod delegate;
mod state;

pub use self::{action::Action, config::Config, delegate::Delegate, state::State};

pub struct Simulation {
    core_id: u32,
    state: State,
}

impl Simulation {
    pub fn new(core_id: u32) -> Result<Self, Error> {
        Ok(Self {
            core_id,
            state: State::setup()?,
        })
    }

    pub fn tick(&mut self, round: u64) -> Result<(), Error> {
        info!(
            core_id = self.core_id,
            round = round,
            "Starting simulation tick"
        );

        let action = self.state.tick()?;

        match action {
            Action::Transfer(transaction) => {
                let response = self.state.delegate.recv_transaction_v0(&transaction)?;

                match response {
                    V0Response::Transaction { outputs } => {
                        let mut index = 0;

                        for atom in transaction.atoms {
                            counter!("mugraph.simulator.atoms_processed").increment(1);

                            if atom.is_input() {
                                counter!("mugraph.simulator.inputs_processed").increment(1);

                                continue;
                            }

                            let asset_id = transaction.asset_ids[atom.asset_id as usize];

                            self.state.recv(asset_id, atom.amount, outputs[index])?;

                            counter!("mugraph.simulator.outputs_received").increment(1);

                            index += 1;
                        }
                    }
                    V0Response::Error { errors } => panic!("{:?}", errors),
                }
            }
        }

        Ok(())
    }
}
