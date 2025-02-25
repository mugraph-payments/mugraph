use std::{fs::OpenOptions, path::PathBuf};

use metrics::counter;
use mugraph_core::{error::Error, types::Signature};
use redb::{
    Builder, Database as Redb, Key, ReadOnlyTable, ReadTransaction, StorageBackend, Table,
    TableDefinition, Value, WriteTransaction, backends::FileBackend,
};

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

#[derive(Debug)]
pub struct Database {
    db: Redb,
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
        })
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
    #[inline]
    pub fn read(&self) -> Result<Read, Error> {
        let result = self.db.begin_read().map(Read).map_err(Error::from)?;
        counter!("mugraph.simulator.database.read").increment(1);

        Ok(result)
    }

    #[tracing::instrument(skip_all)]
    #[inline]
    pub fn write(&self) -> Result<Write, Error> {
        let result = self.db.begin_write().map(Write).map_err(Error::from)?;
        counter!("mugraph.simulator.database.write").increment(1);

        Ok(result)
    }
}
