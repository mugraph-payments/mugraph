mod test_backend;

use std::{
    fs::{self, File},
    path::PathBuf,
};

use mugraph_core::error::Error;
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

    pub fn setup_test() -> Result<Database, Error> {
        Self::setup_with_backend(TestBackend::new(), true)
    }
}
