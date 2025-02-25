use std::sync::atomic::{AtomicU64, Ordering};

use blake3::Hasher;
use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::crypto;

pub static TOTAL_TRANSACTIONS: AtomicU64 = AtomicU64::new(0);
use std::time::Instant;

use mugraph_core::{error::Error, types::*};
use rand::prelude::*;
use tracing::{debug, info, warn};

mod action;
mod config;
mod delegate;
mod state;
mod tick;
pub mod tui;

pub use self::{
    action::Action,
    config::Config,
    delegate::Delegate,
    state::State,
    tick::tick,
};

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
                    counter!("mugraph.simulator.simulated_errors", "reason" => reason).increment(1);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn handle_action(&mut self, action: &Action) -> Result<(), Error> {
        let _start_time = Instant::now();

        match action {
            Action::Refresh(transaction) => {
                info!("Processing transaction");

                let response = self
                    .delegate
                    .client
                    .post(format!("{}/rpc", self.delegate.node_addr))
                    .json(transaction)
                    .send()
                    .map_err(|e| Error::NetworkError {
                        reason: e.to_string(),
                    })?
                    .json::<Response>()
                    .map_err(|e| Error::NetworkError {
                        reason: e.to_string(),
                    })?;

                match response {
                    Response::Transaction { outputs } => {
                        let mut index = 0;

                        for (i, atom) in transaction.atoms.iter().enumerate() {
                            if transaction.is_input(i) {
                                continue;
                            }

                            let asset_id =
                                transaction.asset_ids[atom.asset_id as usize];

                            // Store blinding factors before sending transaction
                            let mut nonce = Hasher::new();
                            nonce.update(asset_id.as_ref());
                            nonce.update(&atom.amount.to_be_bytes());
                            let nonce_hash: Hash = nonce.finalize().into();

                            let blinded = crypto::blind(
                                &mut self.state.rng,
                                nonce_hash.as_ref(),
                            );
                            self.state
                                .blinding_factors
                                .insert(nonce_hash, blinded);

                            self.state.recv(
                                asset_id,
                                atom.amount,
                                outputs[index],
                            )?;

                            index += 1;
                        }

                        counter!("mugraph.simulator.transactions").increment(1);
                        TOTAL_TRANSACTIONS.fetch_add(1, Ordering::Relaxed);
                    }
                    Response::Emit(note) => {
                        return Err(Error::SimulationError {
                            reason: format!(
                                "Unexpected emit response with note: {:?}",
                                note
                            ),
                        });
                    }
                    Response::Error { reason } => {
                        return Err(Error::SimulationError { reason });
                    }
                }
            }
            Action::DoubleRefresh(refresh) => {
                info!("Processing double spend");

                self.delegate
                    .client
                    .post(format!("{}/rpc", self.delegate.node_addr))
                    .json(refresh)
                    .send()
                    .map_err(|e| Error::NetworkError {
                        reason: e.to_string(),
                    })?;

                match self
                    .delegate
                    .client
                    .post(format!("{}/rpc", self.delegate.node_addr))
                    .json(refresh)
                    .send()
                    .map_err(|e| Error::NetworkError {
                        reason: e.to_string(),
                    }) {
                    Ok(_) => {
                        return Err(Error::SimulationError {
                            reason: "Expected redemption to block double spend"
                                .to_string(),
                        });
                    }
                    Err(Error::AlreadySpent { .. }) => {
                        counter!("mugraph.simulator.blocked_double_spent")
                            .increment(1);
                    }
                    Err(e) => return Err(e),
                }

                counter!("mugraph.simulator.double_spends").increment(1);
            }
        }

        Ok(())
    }
}
