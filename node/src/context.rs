use std::sync::Arc;

use color_eyre::eyre::Result;
use mugraph_core::types::Keypair;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{backends::InMemoryBackend, Builder, Database, ReadOnlyTable, TableDefinition};

use crate::database::TestBackend;

// Maps from Commitment to Signature
const TABLE: TableDefinition<[u8; 32], [u8; 32]> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct Context {
    db: Arc<Database>,
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
}

impl Context {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Self> {
        let keypair = Keypair::random(rng);

        let db = Arc::new(Builder::new().create_with_backend(InMemoryBackend::new())?);

        let w = db.begin_write()?;
        {
            let mut t = w.open_table(TABLE)?;
            t.insert(&[0u8; 32], &[0u8; 32])?;
        }
        w.commit()?;

        let rng = ChaCha20Rng::from_rng(rng)?;

        Ok(Self { keypair, db, rng })
    }

    pub fn new_test<R: CryptoRng + RngCore>(rng: &mut R, failure_ratio: f64) -> Result<Self> {
        let keypair = Keypair::random(rng);
        let rng = ChaCha20Rng::from_rng(rng)?;

        let db = Arc::new(
            Builder::new().create_with_backend(TestBackend::new(rng.clone(), failure_ratio))?,
        );

        let w = db.begin_write()?;
        {
            let mut t = w.open_table(TABLE)?;
            t.insert(&[0u8; 32], &[0u8; 32])?;
        }
        w.commit()?;

        Ok(Self { keypair, db, rng })
    }

    pub fn db_read(&self) -> Result<ReadOnlyTable<[u8; 32], [u8; 32]>> {
        let r = self.db.begin_read()?;

        Ok(r.open_table(TABLE)?)
    }
}
