use color_eyre::eyre::Result;
use mugraph_core::{crypto, error::Error, inc, timed, types::*};
use mugraph_node::{database::Database, v0::transaction_v0};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Delegate {
    pub rng: ChaCha20Rng,
    pub db: Database,
    pub keypair: Keypair,
}

impl Delegate {
    pub fn new<R: Rng + CryptoRng>(rng: &mut R, keypair: Keypair) -> Result<Self, Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());

        info!(public_key = %keypair.public_key, "Starting delegate");
        let db = Database::setup_test(&mut rng, None)?;

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

        inc!("stub_notes");

        Ok(note)
    }

    #[inline(always)]
    pub fn recv_transaction_v0(&mut self, tx: &Transaction) -> Result<V0Response, Error> {
        timed!("delegate.transaction_v0", {
            transaction_v0(tx, self.keypair, &self.db)
        })
    }
}
