use std::{cmp::min, collections::HashMap};

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
    pub notes: Vec<Note>,
}

impl State {
    pub fn setup() -> Result<Self, Error> {
        let config = Config::new();
        let mut rng = config.rng();
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let keypair = Keypair::random(&mut rng);
        let mut notes = vec![];
        let mut delegate = Delegate::new(&mut rng, keypair)?;

        for _ in 0..config.notes {
            let idx = rng.gen_range(0..config.assets);

            let asset_id = assets[idx];
            let amount = rng.gen_range(1..1_000_000_000);

            let note = delegate.emit(asset_id, amount)?;

            notes.push(note);
        }

        Ok(Self {
            rng,
            keypair,
            delegate,
            notes,
        })
    }

    pub fn tick(&mut self) -> Result<Action, Error> {
        counter!("mugraph.simulator.state.ticks").increment(1);

        match self.rng.gen_range(0..=1) {
            0 => {
                let input_count = self.rng.gen_range(1..min(8, self.notes.len()));
                let mut transaction = TransactionBuilder::new(GreedyCoinSelection);

                for _ in 0..input_count {
                    let input = self.notes.remove(self.rng.gen_range(0..self.notes.len()));
                    let mut remaining = input.amount;

                    while remaining > 0 {
                        let amount = self.rng.gen_range(1..=remaining);

                        transaction = transaction.output(input.asset_id, amount);

                        remaining -= amount;
                    }

                    transaction = transaction.input(input);
                }

                counter!("mugraph.simulator.state.transfers").increment(1);

                Ok(Action::Split(transaction.build()?))
            }
            1 => {
                let mut transaction = TransactionBuilder::new(GreedyCoinSelection);
                let mut selected_notes = Vec::new();
                let mut selected_indices = Vec::new();

                // Randomly select up to 16 notes
                let num_notes = min(8, self.notes.len());

                while selected_notes.len() < num_notes {
                    let idx = self.rng.gen_range(0..self.notes.len());
                    if !selected_indices.contains(&idx) {
                        selected_indices.push(idx);
                        selected_notes.push(self.notes[idx].clone());
                    }
                }

                let mut asset_groups: HashMap<_, Vec<_>> = HashMap::new();
                for note in selected_notes {
                    asset_groups.entry(note.asset_id).or_default().push(note);
                }

                for (asset_id, notes) in asset_groups.iter() {
                    if notes.len() > 1 {
                        let total_amount: u64 = notes.iter().map(|n| n.amount).sum();

                        for note in notes {
                            transaction = transaction.input(note.clone());
                        }

                        transaction = transaction.output(*asset_id, total_amount);

                        self.notes.retain(|n| !notes.contains(n));

                        // Break after processing one group to avoid overly complex transactions
                        break;
                    }
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

        self.notes.push(note);

        Ok(())
    }
}
