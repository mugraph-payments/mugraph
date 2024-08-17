use std::sync::Arc;

use color_eyre::eyre::Result;
use mugraph_core::types::Keypair;
use rand::prelude::*;
use redb::{backends::InMemoryBackend, Builder, Database, ReadOnlyTable, TableDefinition};

// Maps from Commitment to Signature
const TABLE: TableDefinition<[u8; 32], [u8; 32]> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct Context {
    db: Arc<Database>,
    pub keypair: Keypair,
}

impl Context {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Self> {
        let keypair = Keypair::random(rng);

        let db = Arc::new(Builder::new().create_with_backend(InMemoryBackend::new())?);
        Ok(Self { keypair, db })
    }

    pub fn db_read(&self) -> Result<ReadOnlyTable<[u8; 32], [u8; 32]>> {
        let r = self.db.begin_read()?;

        Ok(r.open_table(TABLE)?)
    }
}
