use std::collections::{HashMap, VecDeque};

use indexmap::{IndexMap, IndexSet};
use metrics::gauge;
use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    crypto::{BlindedPoint, Scalar},
    error::Error,
    types::*,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::{Action, Config, Delegate};

pub struct State {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    pub notes: VecDeque<Note>,
    pub by_asset_id: IndexMap<Hash, IndexSet<u32>>,
    pub blinding_factors: HashMap<Hash, BlindedPoint>,
}

impl State {
    pub fn setup<R: CryptoRng + Rng>(
        rng: &mut R,
        delegate: &mut Delegate,
    ) -> Result<Self, Error> {
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
                .or_insert(IndexSet::from_iter([notes.len() as u32]));

            notes.push_back(note);
        }

        Ok(Self {
            rng: ChaCha20Rng::seed_from_u64(rng.r#gen()),
            keypair: delegate.keypair,
            notes,
            by_asset_id,
            blinding_factors: HashMap::new(),
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn next_action(&mut self) -> Result<Action, Error> {
        gauge!("mugraph.resources", "name" => "available_notes")
            .set(self.notes.len() as f64);

        match self.rng.gen_range(0u32..100) {
            0..45 => self.generate_split(),
            45..90 => self.generate_join(),
            90.. => self.generate_double_spend(),
        }
    }

    #[tracing::instrument(skip_all)]
    fn generate_double_spend(&mut self) -> Result<Action, Error> {
        let mut transaction = RefreshBuilder::new();

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

        Ok(Action::DoubleRefresh(transaction.build()?))
    }

    #[tracing::instrument(skip_all)]
    fn generate_split(&mut self) -> Result<Action, Error> {
        let mut transaction = RefreshBuilder::new();

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

        Ok(Action::Refresh(transaction.build()?))
    }

    #[tracing::instrument(skip_all)]
    fn generate_join(&mut self) -> Result<Action, Error> {
        let mut transaction = RefreshBuilder::new();

        'outer: while transaction.input_count() < MAX_INPUTS {
            let mut found_pair = false;

            for (_, notes) in self.by_asset_id.iter_mut() {
                if self.notes.len() < 2 {
                    break 'outer;
                }

                let a = match notes
                    .pop()
                    .and_then(|x| self.notes.remove(x as usize))
                {
                    Some(a) => a,
                    None => continue,
                };

                let b = match notes
                    .pop()
                    .and_then(|x| self.notes.remove(x as usize))
                {
                    Some(b) => b,
                    None => {
                        // Put 'a' back if we couldn't find a pair
                        notes.insert(self.notes.len() as u32);
                        self.notes.push_back(a);
                        continue;
                    }
                };

                transaction = transaction
                    .output(b.asset_id, b.amount + a.amount)
                    .input(a)
                    .input(b);

                found_pair = true;
                break;
            }

            if !found_pair {
                break;
            }
        }

        if transaction.input_count() == 0 {
            return self.generate_split();
        }

        Ok(Action::Refresh(transaction.build()?))
    }

    #[tracing::instrument(skip_all)]
    pub fn recv(
        &mut self,
        asset_id: Hash,
        amount: u64,
        signature: Blinded<Signature>,
        nonce: Hash,
        blinding_factor: Scalar,
    ) -> Result<(), Error> {
        let note = Note {
            amount,
            delegate: self.keypair.public_key,
            asset_id,
            nonce,
            signature: crypto::unblind_signature(
                &signature,
                &blinding_factor,
                &self.keypair.public_key,
            )?,
        };

        let note_index = self.notes.len() as u32;
        self.notes.push_back(note);

        // Update by_asset_id
        self.by_asset_id
            .entry(asset_id)
            .and_modify(|x: &mut IndexSet<u32>| {
                x.insert(note_index);
            })
            .or_insert_with(|| IndexSet::from_iter([note_index]));

        Ok(())
    }
}
