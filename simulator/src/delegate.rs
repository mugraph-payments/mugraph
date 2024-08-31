use color_eyre::eyre::Result;
use mugraph_core::{crypto, error::Error, types::*};
use mugraph_node::{database::DB, v0::transaction_v0};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::Database;
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
        let failure_rate = rng.gen_range(0.01f64..0.8f64);

        info!(
            "Starting delegate with failure rate {:.2}%",
            failure_rate * 100.0
        );
        let db = DB::setup_test(&mut rng)?;

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

        Ok(note)
    }

    #[inline(always)]
    pub fn recv_transaction_v0(&mut self, tx: &Transaction) -> Result<V0Response, Error> {
        transaction_v0(tx, self.keypair, &self.db)
    }
}
