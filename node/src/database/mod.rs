use std::{fs::OpenOptions, path::PathBuf, sync::atomic::Ordering};

use mugraph_core::{error::Error, types::Signature};
use rand::prelude::*;
use redb::{backends::FileBackend, Builder, Database, StorageBackend, TableDefinition};

mod test_backend;

pub use self::test_backend::*;

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct DB;

impl DB {
    pub fn setup_with_backend<B: StorageBackend>(backend: B) -> Result<Database, Error> {
        let db = Builder::new().create_with_backend(backend)?;

        let w = db.begin_write()?;

        {
            let mut t = w.open_table(NOTES)?;
            t.insert(Signature::zero(), true)?;
        }

        w.commit()?;

        Ok(db)
    }

    pub fn setup(path: impl Into<PathBuf>) -> Result<Database, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.into())?;
        let backend = FileBackend::new(file)?;

        Self::setup_with_backend(backend)
    }

    pub fn setup_test<R: CryptoRng + Rng>(rng: &mut R) -> Result<Database, Error> {
        let backend = TestBackend::new(rng);
        let inject_failures = backend.inject_failures.clone();
        let result = Self::setup_with_backend(backend);
        inject_failures.store(true, Ordering::SeqCst);

        result
    }
}
