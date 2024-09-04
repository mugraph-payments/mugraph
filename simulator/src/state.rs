use std::collections::VecDeque;

use blake3::Hasher;
use indexmap::{IndexMap, IndexSet};
use metrics::gauge;
use mugraph_core::{builder::TransactionBuilder, crypto, error::Error, inc, timed, types::*};
use priority_queue::PriorityQueue;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::{Action, Config, Delegate};

pub struct State {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    pub notes: VecDeque<Note>,
    pub by_asset_id: IndexMap<Hash, IndexSet<u32>>,
    pub scheduled_actions: PriorityQueue<u64, Action>,
}

impl State {
    pub fn setup(mut delegate: Delegate) -> Result<Self, Error> {
        let config = Config::new();
        let mut rng = config.rng();
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut notes = VecDeque::with_capacity(config.notes);
        let mut by_asset_id = IndexMap::new();

        for _ in 0..config.notes {
            let idx = rng.gen_range(0..config.assets);

            let asset_id = assets[idx];
            let amount = rng.gen_range(1..u64::MAX / 2);

            let note = delegate.emit(asset_id, amount)?;

            by_asset_id
                .entry(note.asset_id)
                .and_modify(|x: &mut IndexSet<u32>| {
                    x.insert(notes.len() as u32);
                })
                .or_insert(vec![notes.len() as u32].into_iter().collect());

            notes.push_back(note);
        }

        Ok(Self {
            rng,
            keypair: delegate.keypair,
            notes,
            by_asset_id,
            scheduled_actions: PriorityQueue::new(),
        })
    }

    pub fn schedule(&mut self, round: u64, action: Action) {
        self.scheduled_actions.push(round, action);
    }

    pub fn next_action(&mut self, round: u64) -> Result<Action, Error> {
        gauge!("mugraph.resources", "name" => "available_notes").set(self.notes.len() as f64);

        if self.notes.len() == 0 {
            return timed!("state.next_action.split", { self.generate_split() });
        }

        match self.scheduled_actions.pop() {
            Some((action_round, action)) if action_round <= round => {
                return Ok(action);
            }
            Some((round, action)) => {
                self.scheduled_actions.push(round, action);
            }
            _ => {}
        }

        match self.rng.gen_range(0..100u32) {
            0..45 => timed!("state.next_action.split", { self.generate_split() }),
            45..90 => timed!("state.next_action.join", { self.generate_join() }),
            90.. => timed!("state.next_action.double_spend", {
                self.generate_double_spend()
            }),
        }
    }

    fn generate_double_spend(&mut self) -> Result<Action, Error> {
        let mut transaction = TransactionBuilder::new();

        match self.notes.pop_front() {
            Some(input) => {
                transaction = transaction
                    .output(input.asset_id, input.amount)
                    .input(input);
            }
            None => {
                return Err(Error::ServerError {
                    reason: "no notes".into(),
                });
            }
        }

        inc!("state.double_spends");
        Ok(Action::DoubleSpend(transaction.build()?))
    }

    fn generate_split(&mut self) -> Result<Action, Error> {
        let mut transaction = TransactionBuilder::new();

        while transaction.output_count() < MAX_OUTPUTS {
            let input = match self.notes.pop_front() {
                Some(input) => input,
                None => {
                    break;
                }
            };

            if input.amount > 2 {
                let rem = input.amount % 2;
                let (a, b) = (input.amount / 2, input.amount / 2 + rem);

                transaction = transaction
                    .output(input.asset_id, a)
                    .output(input.asset_id, b);
            } else {
                transaction = transaction.output(input.asset_id, input.amount);
            }

            transaction = transaction.input(input);

            inc!("state.splits");
        }

        if transaction.input_count() == 0 {
            return Err(Error::ServerError {
                reason: "no notes".into(),
            });
        }

        Ok(Action::Transaction(transaction.build()?))
    }

    fn generate_join(&mut self) -> Result<Action, Error> {
        let mut transaction = TransactionBuilder::new();

        for (_, notes) in self.by_asset_id.iter_mut() {
            if self.notes.is_empty() {
                break;
            }

            if notes.len() < 2 {
                continue;
            }

            let a = self
                .notes
                .remove(self.rng.gen_range(0..self.notes.len()))
                .unwrap();
            let b = self
                .notes
                .remove(self.rng.gen_range(0..self.notes.len()))
                .unwrap();

            transaction = transaction
                .output(a.asset_id, a.amount + b.amount)
                .input(a)
                .input(b);

            break;
        }

        if transaction.input_count() == 0 {
            return Err(Error::ServerError {
                reason: "no notes".into(),
            });
        }

        inc!("state.joins");

        Ok(Action::Transaction(transaction.build()?))
    }

    pub fn recv(
        &mut self,
        asset_id: Hash,
        amount: u64,
        signature: Blinded<Signature>,
    ) -> Result<(), Error> {
        inc!("state.notes_received");

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
