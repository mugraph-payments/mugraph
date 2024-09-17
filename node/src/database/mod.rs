use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use mugraph_core::{error::Error, inc, types::Signature, utils::timed};
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
    #[timed]
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
    #[timed]
    pub fn open_table<K: Key, V: Value>(
        &self,
        table: TableDefinition<K, V>,
    ) -> Result<Table<K, V>, Error> {
        inc!("database.write.open_table");
        Ok(self.0.open_table(table)?)
    }

    #[timed]
    pub fn commit(self) -> Result<(), Error> {
        inc!("database.write.commit");
        Ok(self.0.commit()?)
    }
}

impl Database {
    pub fn setup<R: CryptoRng + Rng>(rng: &mut R, path: impl Into<PathBuf>) -> Result<Self, Error> {
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
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
        })
    }

    pub fn setup_test<R: CryptoRng + Rng>(
        rng: &mut R,
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
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

    #[timed]
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

        inc!("database.reopen");

        Ok(())
    }

    fn setup_with_backend<B: StorageBackend>(
        backend: B,
        should_setup: bool,
    ) -> Result<Redb, Error> {
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

    #[timed]
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
                inc!("database.read");
                v
            }
        }
    }

    #[timed]
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
