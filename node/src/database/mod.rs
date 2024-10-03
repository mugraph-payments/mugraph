use std::{fs::OpenOptions, path::PathBuf};

use metrics::counter;
use mugraph_core::{error::Error, types::Signature};
use redb::{
    backends::FileBackend, Builder, Database as Redb, Key, ReadOnlyTable, ReadTransaction,
    StorageBackend, Table, TableDefinition, Value, WriteTransaction,
};

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

#[derive(Debug)]
pub struct Database {
    db: Redb,
    file_path: PathBuf,
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
            db: Self::setup_with_backend(backend, true)?,
            file_path: path,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn reopen(&mut self) -> Result<(), Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.file_path)?;
        let backend = FileBackend::new(file)?;

        self.db = Self::setup_with_backend(backend, false)?;

        counter!("mugraph.simulator.database.reopen").increment(1);

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
