use std::collections::{BTreeMap, VecDeque};

use blake3::Hasher;
use metrics::{counter, gauge};
use mugraph_core::{builder::TransactionBuilder, crypto, error::Error, timed, types::*};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::info;

use crate::{Action, Config, Delegate};

pub struct State {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    pub notes: VecDeque<Note>,
}

impl State {
    pub fn setup(mut delegate: Delegate) -> Result<Self, Error> {
        let config = Config::new();
        let mut rng = config.rng();
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut notes = VecDeque::with_capacity(config.notes);

        for _ in 0..config.notes {
            let idx = rng.gen_range(0..config.assets);

            let asset_id = assets[idx];
            let amount = rng.gen_range(1..1_000_000_000);

            let note = delegate.emit(asset_id, amount)?;

            notes.push_back(note);
        }

        Ok(Self {
            rng,
            keypair: delegate.keypair,
            notes,
        })
    }

    pub fn next_action(&mut self) -> Result<Action, Error> {
        gauge!("state.note_count").set(self.notes.len() as f64);

        match self.rng.gen_bool(0.5) {
            true => timed!("state.next.split", {
                let input_count = self.rng.gen_range(1..4);
                let mut transaction = TransactionBuilder::new();

                for _ in 0..input_count {
                    let input = match self.notes.pop_front() {
                        Some(input) => input,
                        None => break,
                    };

                    if input.amount > 2 {
                        let rem = input.amount % 2;
                        let (a, b) = (input.amount / 2, input.amount / 2 + rem);

                        transaction = transaction
                            .output(input.asset_id, a)
                            .output(input.asset_id, b);
                    }

                    transaction = transaction.input(input);
                }

                info!("Split generated");
                counter!("state.splits").increment(1);

                Ok(Action::Split(transaction.build()?))
            }),
            false => timed!("state.next.join", {
                let mut transaction = TransactionBuilder::new();
                let mut outputs: BTreeMap<Hash, u64> = BTreeMap::new();

                for _ in 0..4 {
                    let note = match self.notes.pop_front() {
                        Some(n) => n,
                        _ => {
                            return Err(Error::ServerError {
                                reason: "No notes available".to_string(),
                            })
                        }
                    };

                    outputs
                        .entry(note.asset_id)
                        .and_modify(|x| *x += note.amount)
                        .or_default();

                    transaction = transaction.input(note);
                }

                for (asset_id, amount) in outputs {
                    transaction = transaction.output(asset_id, amount);
                }

                counter!("state.joins").increment(1);

                Ok(Action::Join(transaction.build()?))
            }),
        }
    }

    pub fn recv(
        &mut self,
        asset_id: Hash,
        amount: u64,
        signature: Blinded<Signature>,
    ) -> Result<(), Error> {
        counter!("state.notes_received").increment(1);
        let mut nonce = Hasher::new();
        nonce.update(asset_id.as_ref());
        nonce.update(&amount.to_be_bytes());
        nonce.update(signature.0.as_ref());

        let note = Note {
            amount,
            delegate: self.keypair.public_key,
            asset_id,
            nonce: nonce.finalize().into(),
            signature: crypto::unblind_signature(
                &signature,
                &crypto::blind(&mut self.rng, &[]).factor,
                &self.keypair.public_key,
            )?,
        };

        self.notes.push_back(note);

        Ok(())
    }
}
