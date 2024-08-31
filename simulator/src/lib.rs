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
use tracing::{error, info};

mod action;
mod agents;
mod config;
mod state;

pub use self::{
    action::Action,
    agents::{delegate::Delegate, user::User},
    config::Config,
    state::State,
};

pub struct Simulation {
    state: State,
}

impl Simulation {
    pub fn new(core_id: u32) -> Result<Self, Error> {
        Ok(Self {
            state: State::setup()?,
        })
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        let action = self.state.tick()?;

        match action {
            Action::Transfer(transaction) => {
                let response = self.state.delegate.recv_transaction_v0(transaction)?;

                match response {
                    V0Response::Transaction { outputs } => {
                        let index = 0;

                        for atom in transaction.atoms {
                            if atom.is_input() {
                                continue;
                            }

                            self.state.recv(atom.asset_id, atom.amount, outputs[index]);

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
