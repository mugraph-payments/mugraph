mod test_backend;

use std::{
    fs::{self, File},
    path::PathBuf,
    sync::atomic::Ordering,
};

use mugraph_core::error::Error;
use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use redb::{backends::FileBackend, Builder, Database, StorageBackend, TableDefinition};
pub use test_backend::*;

pub const TABLE: TableDefinition<[u8; 32], bool> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct DB;

impl DB {
    pub fn setup_with_backend<B: StorageBackend>(
        backend: B,
        do_setup: bool,
    ) -> Result<Database, Error> {
        let db = Builder::new().create_with_backend(backend)?;

        if do_setup {
            let w = db.begin_write()?;

            {
                let mut t = w.open_table(TABLE)?;
                t.insert(&[0u8; 32], false)?;
            }

            w.commit()?;
        }

        Ok(db)
    }

    pub fn setup(path: impl Into<PathBuf>) -> Result<Database, Error> {
        let path = path.into();
        let backend = FileBackend::new(File::open(&path)?)?;

        Self::setup_with_backend(backend, !fs::exists(&path)?)
    }

    pub fn setup_test<R: CryptoRng + Rng>(rng: &mut R) -> Result<Database, Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());
        let failure_rate = rng.gen_range(0.0..1.0);
        let backend = TestBackend::new(rng, failure_rate);
        let enable_failures = backend.enable_failures.clone();

        let db = Self::setup_with_backend(backend, true)?;

        enable_failures.store(true, Ordering::SeqCst);

        Ok(db)
    }
}
