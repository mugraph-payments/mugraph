use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::Database;

use crate::{config::Config, database::DB};

#[derive(Debug, Clone)]
pub struct Context {
    pub config: Config,
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
}

impl Context {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Self> {
        let config = Config::new();
        let keypair = Keypair::random(rng);
        let rng = ChaCha20Rng::seed_from_u64(rng.gen());

        Ok(Self {
            config,
            keypair,
            rng,
        })
    }

    pub fn db(&mut self) -> Result<Database, Error> {
        let db = if self.config.under_test.unwrap_or(false) {
            DB::setup_test(&mut self.rng)
        } else {
            DB::setup("db")
        };

        Ok(db?.inner)
    }
}
