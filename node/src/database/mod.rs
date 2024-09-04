use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    sync::{atomic::Ordering, Arc, RwLock},
};

use mugraph_core::{error::Error, inc, types::Signature};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{
    backends::FileBackend, Builder, Database as Redb, Key, ReadOnlyTable, ReadTransaction,
    StorageBackend, Table, TableDefinition, Value, WriteTransaction,
};

mod test_backend;

pub use self::test_backend::*;

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

#[derive(Debug, Clone)]
pub struct Database {
    mode: Mode,
    rng: ChaCha20Rng,
    db: Arc<RwLock<Redb>>,
}

#[derive(Debug, Clone)]
pub enum Mode {
    File { path: PathBuf },
    Test { path: PathBuf },
}

#[repr(transparent)]
pub struct Read(ReadTransaction);

impl Read {
    #[inline(always)]
    pub fn open_table<K: Key, V: Value>(
        &self,
        table: TableDefinition<K, V>,
    ) -> Result<ReadOnlyTable<K, V>, Error> {
        inc!("redb.read.open_table");
        Ok(self.0.open_table(table)?)
    }
}

#[repr(transparent)]
pub struct Write(WriteTransaction);

impl Write {
    #[inline(always)]
    pub fn open_table<K: Key, V: Value>(
        &self,
        table: TableDefinition<K, V>,
    ) -> Result<Table<K, V>, Error> {
        inc!("database.write.open_table");
        Ok(self.0.open_table(table)?)
    }

    #[inline(always)]
    pub fn commit(self) -> Result<(), Error> {
        inc!("redb.write.commit");
        Ok(self.0.commit()?)
    }
}

impl Database {
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

    pub fn setup<R: CryptoRng + Rng>(rng: &mut R, path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        let exists = fs::exists(&path).unwrap_or(false);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        let backend = FileBackend::new(file)?;

        Ok(Self {
            mode: Mode::File { path },
            db: Arc::new(RwLock::new(Self::setup_with_backend(backend, !exists)?)),
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
        })
    }

    pub fn setup_test<R: CryptoRng + Rng>(
        rng: &mut R,
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let exists = path.is_some();
        let (inject_failures, backend) = TestBackend::new(rng, path)?;
        let path = backend.path.clone();

        inject_failures.store(true, Ordering::SeqCst);
        let db = Self::setup_with_backend(backend, !exists)?;

        Ok(Self {
            mode: Mode::Test { path },
            db: Arc::new(RwLock::new(db)),
            rng: ChaCha20Rng::seed_from_u64(rng.gen()),
        })
    }

    pub fn check_integrity(&self) -> Result<(), Error> {
        match self.mode {
            Mode::File { ref path } => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)?;
                let backend = FileBackend::new(file)?;

                let result = Self::setup_with_backend(backend, false)?;

                let mut w = self.db.write()?;
                *w = result;
            }
            Mode::Test { ref path } => {
                let mut rng = self.rng.clone();

                let (inject_failures, backend) = TestBackend::new(&mut rng, Some(path.clone()))?;
                let result = Self::setup_with_backend(backend, false)?;

                let mut w = self.db.write()?;
                *w = result;

                inject_failures.store(true, Ordering::SeqCst);
            }
        }

        inc!("database.reconnections");

        Ok(())
    }

    pub fn read(&mut self) -> Result<Read, Error> {
        inc!("database.read");

        let result = {
            let db = self.db.read()?;

            db.begin_read().map(Read).map_err(Error::from)
        };

        match result {
            Err(Error::StorageError { reason, .. })
                if reason.to_lowercase().contains("previous i/o error") =>
            {
                self.check_integrity()?;
                self.read()
            }
            v => v,
        }
    }

    pub fn write(&mut self) -> Result<Write, Error> {
        inc!("database.write");

        let result = {
            let db = self.db.read()?;

            db.begin_write().map(Write).map_err(Error::from)
        };

        match result {
            Err(Error::StorageError { reason, .. })
                if reason.to_lowercase().contains("previous i/o error") =>
            {
                self.check_integrity()?;
                self.write()
            }
            v => v,
        }
    }
}
