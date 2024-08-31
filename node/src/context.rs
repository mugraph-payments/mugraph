use std::sync::atomic::Ordering;

use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{backends::InMemoryBackend, Builder, Database, ReadOnlyTable, TableDefinition};

use crate::database::TestBackend;

// Maps from Signature to bool
pub const TABLE: TableDefinition<[u8; 32], bool> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct Context {
    pub db: Database,
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
}

impl Context {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Self> {
        let keypair = Keypair::random(rng);

        let db = Builder::new().create_with_backend(InMemoryBackend::new())?;

        let w = db.begin_write()?;
        {
            let mut t = w.open_table(TABLE)?;
            t.insert(&[0u8; 32], false)?;
        }
        w.commit()?;

        let rng = ChaCha20Rng::seed_from_u64(rng.gen());

        Ok(Self { keypair, db, rng })
    }

    pub fn new_test<R: CryptoRng + RngCore>(rng: &mut R, failure_rate: f64) -> Result<Self, Error> {
        let keypair = Keypair::random(rng);
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());

        let backend = TestBackend::new(ChaCha20Rng::seed_from_u64(rng.gen()), failure_rate);
        let enable_failures = backend.enable_failures.clone();
        let db = Builder::new().create_with_backend(backend)?;

        let w = db.begin_write()?;
        {
            let mut t = w.open_table(TABLE)?;
            t.insert(&[0u8; 32], false)?;
        }
        w.commit()?;

        enable_failures.store(true, Ordering::Relaxed);

        Ok(Self { keypair, db, rng })
    }

    pub fn db_read(&self) -> Result<ReadOnlyTable<[u8; 32], bool>> {
        let r = self.db.begin_read()?;

        Ok(r.open_table(TABLE)?)
    }
}
