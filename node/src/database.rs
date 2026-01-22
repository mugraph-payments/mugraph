use std::{fs::OpenOptions, path::PathBuf};

use metrics::counter;
use mugraph_core::{
    error::Error,
    types::{CardanoWallet, DepositRecord, Signature, UtxoRef, WithdrawalKey, WithdrawalRecord},
};
use redb::{
    Builder,
    Database as Redb,
    Key,
    ReadOnlyTable,
    ReadTransaction,
    ReadableDatabase,
    StorageBackend,
    Table,
    TableDefinition,
    Value,
    WriteTransaction,
    backends::FileBackend,
};

pub const NOTES: TableDefinition<Signature, bool> = TableDefinition::new("notes");

/// Schema version key for database migrations
pub const SCHEMA_VERSION: TableDefinition<&str, u64> = TableDefinition::new("schema_version");

/// Cardano wallet data (single row with key "wallet")
pub const CARDANO_WALLET: TableDefinition<&str, CardanoWallet> =
    TableDefinition::new("cardano_wallet");

/// Deposits indexed by UTxO reference (tx_hash[32] + index[2])
pub const DEPOSITS: TableDefinition<UtxoRef, DepositRecord> = TableDefinition::new("deposits");

/// Withdrawals indexed by network[1] + tx_hash[32]
pub const WITHDRAWALS: TableDefinition<WithdrawalKey, WithdrawalRecord> =
    TableDefinition::new("withdrawals");

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
    ) -> Result<Table<'_, K, V>, Error> {
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

    fn setup_with_backend<B: StorageBackend>(backend: B, should_setup: bool) -> Result<Redb, Error> {
        let db = Builder::new().create_with_backend(backend)?;

        if should_setup {
            let w = db.begin_write()?;

            {
                let mut t = w.open_table(NOTES)?;
                t.insert(Signature::zero(), true)?;
            }

            {
                let mut t = w.open_table(SCHEMA_VERSION)?;
                t.insert("version", 1)?;
            }

            w.commit()?;
        }

        Ok(db)
    }

    /// Run database migrations to create new tables
    pub fn migrate(&self) -> Result<(), Error> {
        let w = self.db.begin_write()?;

        // Create CARDANO_WALLET table if it doesn't exist
        {
            let _ = w.open_table(CARDANO_WALLET)?;
        }

        // Create DEPOSITS table if it doesn't exist
        {
            let _ = w.open_table(DEPOSITS)?;
        }

        // Create WITHDRAWALS table if it doesn't exist
        {
            let _ = w.open_table(WITHDRAWALS)?;
        }

        // Update schema version
        {
            let mut t = w.open_table(SCHEMA_VERSION)?;
            t.insert("version", 2)?;
        }

        w.commit()?;
        Ok(())
    }

    /// Get current schema version
    pub fn schema_version(&self) -> Result<u64, Error> {
        let r = self.db.begin_read()?;
        let t = r.open_table(SCHEMA_VERSION)?;
        match t.get("version")? {
            Some(v) => Ok(v.value()),
            None => Ok(0),
        }
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
