use std::sync::Arc;

use color_eyre::eyre::Result;
use metrics::counter;
use mugraph_core::{crypto, error::Error, timed, types::*};
use mugraph_node::{database::DB, v0::transaction_v0};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::Database;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Delegate {
    pub rng: ChaCha20Rng,
    pub db: Arc<Database>,
    pub keypair: Keypair,
}

impl Delegate {
    pub fn new<R: Rng + CryptoRng>(rng: &mut R, keypair: Keypair) -> Result<Self, Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());

        info!(public_key = %keypair.public_key, "Starting delegate");
        let db = DB::setup_test(&mut rng)?.into();

        counter!("mugraph.simulator.delegates_spawned").increment(1);

        Ok(Self { db, rng, keypair })
    }

    pub fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note, Error> {
        let mut note = Note {
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut self.rng),
            amount,
            signature: Signature::default(),
        };

        let blind = crypto::blind_note(&mut self.rng, &note);
        let signed = crypto::sign_blinded(&self.keypair.secret_key, &blind.point);
        note.signature =
            crypto::unblind_signature(&signed, &blind.factor, &self.keypair.public_key)?;

        counter!("mugraph.simulator.stub_notes_emitted").increment(1);

        Ok(note)
    }

    #[inline(always)]
    pub fn recv_transaction_v0(&mut self, tx: &Transaction) -> Result<V0Response, Error> {
        timed!("mugraph.simulator.delegate.transaction_v0", {
            transaction_v0(tx, self.keypair, &self.db)
        })
    }
}
