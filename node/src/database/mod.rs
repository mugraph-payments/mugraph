use std::{fs::OpenOptions, path::PathBuf};

use metrics::counter;
use mugraph_core::{error::Error, types::Signature};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{
    backends::FileBackend, Builder, Database as Redb, Key, ReadOnlyTable, ReadTransaction,
    StorageBackend, Table, TableDefinition, Value, WriteTransaction,
};

mod test_backend;

pub use self::test_backend::*;

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

#[derive(Debug)]
pub struct Database {
    mode: Mode,
    rng: ChaCha20Rng,
    db: Redb,
}

#[derive(Debug, Clone)]
pub enum Mode {
    File { path: PathBuf },
    Test { path: PathBuf },
}

#[repr(transparent)]
pub struct Read(ReadTransaction);

impl Read {
    #[tracing::instrument(skip_all)]
    pub fn open_table<K: Key, V: Value>(
        &self,
        table: TableDefinition<K, V>,
    ) -> Result<ReadOnlyTable<K, V>, Error> {
        Ok(self.0.open_table(table)?)
    }
}

#[repr(transparent)]
pub struct Write(WriteTransaction);

impl Write {
    #[tracing::instrument(skip_all)]
    pub fn open_table<K: Key, V: Value>(
        &self,
        table: TableDefinition<K, V>,
    ) -> Result<Table<K, V>, Error> {
        counter!("mugraph.simulator.database.write.open_table").increment(1);
        Ok(self.0.open_table(table)?)
    }

    #[tracing::instrument(skip_all)]
    pub fn commit(self) -> Result<(), Error> {
        counter!("mugraph.simulator.database.write.commit").increment(1);
        Ok(self.0.commit()?)
    }
}

impl Database {
    pub fn setup(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        let backend = FileBackend::new(file)?;

        Ok(Self {
            db: Self::setup_with_backend(backend, !path.exists())?,
            mode: Mode::File { path },
            rng: ChaCha20Rng::seed_from_u64(thread_rng().gen()),
        })
    }

    pub fn setup_test<R: CryptoRng + Rng>(rng: &mut R, path: Option<PathBuf>) -> Result<Self, Error> {
        let exists = path.is_some();
        let backend = TestBackend::new(rng, path)?;
        let path = backend.path.clone();

        let db = Self::setup_with_backend(backend, !exists)?;

        Ok(Self {
            mode: Mode::Test { path },
            db,
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn reopen(&mut self) -> Result<(), Error> {
        match self.mode {
            Mode::File { ref path } => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)?;
                let backend = FileBackend::new(file)?;

                self.db = Self::setup_with_backend(backend, false)?;
            }
            Mode::Test { ref path } => {
                let backend = TestBackend::new(&mut self.rng.clone(), Some(path.clone()))?;
                self.db = Self::setup_with_backend(backend, false)?;
            }
        }

        counter!("mugraph.simulator.database.reopen").increment(1);

        Ok(())
    }

    fn setup_with_backend<B: StorageBackend>(backend: B, should_setup: bool) -> Result<Redb, Error> {
        let db = Builder::new().create_with_backend(backend)?;

        if should_setup {
            let w = db.begin_write()?;

            {
                let mut t = w.open_table(NOTES)?;
                t.insert(Signature::zero(), true)?;
            }

            w.commit()?;
        }

        Ok(db)
    }

    #[tracing::instrument(skip_all)]
    pub fn read(&mut self) -> Result<Read, Error> {
        let result = { self.db.begin_read().map(Read).map_err(Error::from) };

        match result {
            Err(Error::StorageError { reason, .. })
                if reason.to_lowercase().contains("previous i/o error") =>
            {
                self.reopen()?;
                self.read()
            }
            v => {
                counter!("mugraph.simulator.database.read").increment(1);
                v
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn write(&mut self) -> Result<Write, Error> {
        let result = { self.db.begin_write().map(Write).map_err(Error::from) };

        match result {
            Err(Error::StorageError { reason, .. })
                if reason.to_lowercase().contains("previous i/o error") =>
            {
                self.reopen()?;
                self.write()
            }
            v => v,
        }
    }
}
