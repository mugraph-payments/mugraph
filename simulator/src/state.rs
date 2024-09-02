use std::{
    cmp::min,
    collections::{HashMap, HashSet},
};

use metrics::counter;
use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    error::Error,
    types::*,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::{Action, Config, Delegate};

pub struct State {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    pub delegate: Delegate,
    pub notes: HashSet<Note>,
}

impl State {
    pub fn setup() -> Result<Self, Error> {
        let config = Config::new();
        let mut rng = config.rng();
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let keypair = Keypair::random(&mut rng);
        let mut notes = HashSet::new();
        let mut delegate = Delegate::new(&mut rng, keypair)?;
        let mut asset_groups: HashMap<Hash, Vec<u32>> = HashMap::new();

        for _ in 0..config.notes {
            let idx = rng.gen_range(0..config.assets);

            let asset_id = assets[idx];
            let amount = rng.gen_range(1..1_000_000_000);

            let note = delegate.emit(asset_id, amount)?;

            asset_groups
                .entry(asset_id)
                .and_modify(|x| x.push((notes.len() + 1) as u32))
                .or_default();
            notes.insert(note);
        }

        Ok(Self {
            rng,
            keypair,
            delegate,
            notes,
        })
    }

    pub fn next(&mut self) -> Result<Action, Error> {
        counter!("mugraph.simulator.state.ticks").increment(1);

        match self.rng.gen_range(0..=1) {
            0 => {
                let input_count = self.rng.gen_range(1..min(8, self.notes.len()));
                let mut transaction = TransactionBuilder::new(GreedyCoinSelection);

                for _ in 0..input_count {
                    let notes = self.notes.clone();
                    let input = match notes.iter().choose(&mut self.rng) {
                        Some(v) => v,
                        None => continue,
                    };
                    let mut remaining = input.amount;

                    while remaining > 0 {
                        let amount = self.rng.gen_range(1..=remaining);

                        transaction = transaction.output(input.asset_id, amount);

                        remaining -= amount;
                    }

                    self.notes.remove(input);
                    transaction = transaction.input(input.clone());
                }

                counter!("mugraph.simulator.state.transfers").increment(1);

                Ok(Action::Split(transaction.build()?))
            }
            1 => {
                let mut transaction = TransactionBuilder::new(GreedyCoinSelection);
                let mut selected = vec![];
                let mut outputs: HashMap<Hash, u64> = HashMap::new();

                let notes = self.notes.clone();

                for note in notes {
                    if selected.len() >= 16 {
                        break;
                    }

                    outputs
                        .entry(note.asset_id)
                        .and_modify(|x| *x += note.amount)
                        .or_default();

                    transaction = transaction.input(note.clone());
                    selected.push(note);
                }

                for note in selected {
                    self.notes.remove(&note);
                }

                for (asset_id, amount) in outputs {
                    transaction = transaction.output(asset_id, amount);
                }

                counter!("mugraph.simulator.state.joins").increment(1);

                Ok(Action::Join(transaction.build()?))
            }
            _ => unreachable!(),
        }
    }

    pub fn recv(
        &mut self,
        asset_id: Hash,
        amount: u64,
        signature: Blinded<Signature>,
    ) -> Result<(), Error> {
        counter!("mugraph.simulator.state.notes_received").increment(1);

        let note = Note {
            amount,
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut self.rng),
            signature: crypto::unblind_signature(
                &signature,
                &crypto::blind(&mut self.rng, &[]).factor,
                &self.keypair.public_key,
            )?,
        };

        self.notes.insert(note);

        Ok(())
    }
}
