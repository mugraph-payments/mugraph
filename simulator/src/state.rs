use std::collections::VecDeque;

use blake3::Hasher;
use indexmap::{IndexMap, IndexSet};
use metrics::gauge;
use mugraph_core::{builder::TransactionBuilder, crypto, error::Error, types::*, utils::timed};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::{Action, Config, Delegate};

pub struct State {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    pub notes: VecDeque<Note>,
    pub by_asset_id: IndexMap<Hash, IndexSet<u32>>,
}

impl State {
    pub fn setup<R: CryptoRng + Rng>(rng: &mut R, mut delegate: Delegate) -> Result<Self, Error> {
        let config = Config::new();
        let assets = (0..config.assets)
            .map(|_| Hash::random(rng))
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
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
            keypair: delegate.keypair,
            notes,
            by_asset_id,
        })
    }

    #[timed]
    pub fn next_action(&mut self) -> Result<Action, Error> {
        gauge!("mugraph.resources", "name" => "available_notes").set(self.notes.len() as f64);

        if self.notes.is_empty() {
            return self.generate_split();
        }

        if self.notes.is_empty() {
            return self.generate_split();
        }

        match self.rng.gen_range(0u32..100) {
            0..45 => self.generate_split(),
            45..90 => self.generate_join(),
            90.. => self.generate_double_spend(),
        }
    }

    #[timed]
    fn generate_double_spend(&mut self) -> Result<Action, Error> {
        let mut transaction = TransactionBuilder::new();

        match self.notes.pop_front() {
            Some(input) => {
                transaction = transaction
                    .output(input.asset_id, input.amount)
                    .input(input);
            }
            None => {
                return self.generate_split();
            }
        }

        Ok(Action::DoubleSpend(transaction.build()?))
    }

    #[timed]
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
        }

        if transaction.input_count() == 0 {
            return Err(Error::ServerError {
                reason: "no notes".into(),
            });
        }

        Ok(Action::Transaction(transaction.build()?))
    }

    #[timed]
    fn generate_join(&mut self) -> Result<Action, Error> {
        let mut transaction = TransactionBuilder::new();

        'outer: while transaction.input_count() < MAX_INPUTS {
            for (_, notes) in self.by_asset_id.iter_mut() {
                if self.notes.is_empty() {
                    break 'outer;
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
            }
        }

        if transaction.input_count() == 0 {
            return self.generate_split();
        }

        Ok(Action::Transaction(transaction.build()?))
    }

    #[timed]
    pub fn recv(
        &mut self,
        asset_id: Hash,
        amount: u64,
        signature: Blinded<Signature>,
    ) -> Result<(), Error> {
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
