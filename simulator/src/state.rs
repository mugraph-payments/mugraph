use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    error::Error,
    types::*,
};
use mugraph_node::context::Context;
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
        let context = Context::new(&mut rng)?;
        let mut delegate = Delegate::new(&mut rng, context)?;

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
        match self.rng.gen_range(0..=0) {
            0 => {
                let input_count = self.rng.gen_range(1..self.notes.len());
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

                Ok(Action::Transfer(transaction.build()?))
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
