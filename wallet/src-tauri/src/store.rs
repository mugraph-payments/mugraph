use std::path::Path;

use mugraph_core::types::{Hash, Note, PublicKey};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

const CONFIG_GLOBAL: TableDefinition<&str, &str> =
    TableDefinition::new("config_global");
const PROVIDER_CONFIG: TableDefinition<&str, &str> =
    TableDefinition::new("provider_config");
const NODE_CONFIG: TableDefinition<&str, &str> =
    TableDefinition::new("node_config");
const KEYPAIR: TableDefinition<&str, &[u8]> = TableDefinition::new("keypair");
const CARDANO_KEYPAIR: TableDefinition<&str, &[u8]> =
    TableDefinition::new("cardano_keypair");
const DELEGATE_INFO: TableDefinition<&str, &[u8]> =
    TableDefinition::new("delegate_info");
const NOTES: TableDefinition<&str, &[u8]> = TableDefinition::new("notes");
const ACTIVITY: TableDefinition<&str, &[u8]> = TableDefinition::new("activity");
const BLINDING_FACTORS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("blinding_factors");
const OFFCHAIN_REQUESTS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("offchain_requests");
const CARDANO_UTXOS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("cardano_utxos");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    Available,
    Spent,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredNote {
    pub note: Note,
    pub status: NoteStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: String,
    pub kind: String,
    pub timestamp: u64,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct OrphanedBlindingFactor {
    pub key: String,
    pub factor_bytes: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("storage error: {0}")]
    Storage(#[from] redb::StorageError),
    #[error("table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

pub struct Store {
    db: Database,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let db = Database::create(path)?;

        // Create all tables on first open
        let txn = db.begin_write()?;
        {
            txn.open_table(CONFIG_GLOBAL)?;
            txn.open_table(PROVIDER_CONFIG)?;
            txn.open_table(NODE_CONFIG)?;
            txn.open_table(KEYPAIR)?;
            txn.open_table(CARDANO_KEYPAIR)?;
            txn.open_table(DELEGATE_INFO)?;
            txn.open_table(NOTES)?;
            txn.open_table(ACTIVITY)?;
            txn.open_table(BLINDING_FACTORS)?;
            txn.open_table(OFFCHAIN_REQUESTS)?;
            txn.open_table(CARDANO_UTXOS)?;
        }
        txn.commit()?;

        Ok(Self { db })
    }

    // --- Config ---

    pub fn set_config(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(CONFIG_GLOBAL)?;
            table.insert(key, value)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(CONFIG_GLOBAL)?;
        Ok(table.get(key)?.map(|v| v.value().to_string()))
    }

    // --- Provider Config ---

    pub fn set_provider_config(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(PROVIDER_CONFIG)?;
            table.insert(key, value)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_provider_config(
        &self,
        key: &str,
    ) -> Result<Option<String>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(PROVIDER_CONFIG)?;
        Ok(table.get(key)?.map(|v| v.value().to_string()))
    }

    // --- Node Config ---

    pub fn set_node_url(
        &self,
        network: &str,
        url: &str,
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(NODE_CONFIG)?;
            table.insert(network, url)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_node_url(
        &self,
        network: &str,
    ) -> Result<Option<String>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(NODE_CONFIG)?;
        Ok(table.get(network)?.map(|v| v.value().to_string()))
    }

    // --- Keypair ---

    pub fn set_keypair_bytes(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(KEYPAIR)?;
            table.insert(key, value)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_keypair_bytes(
        &self,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(KEYPAIR)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }

    // --- Cardano Keypair ---

    pub fn set_cardano_keypair_bytes(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(CARDANO_KEYPAIR)?;
            table.insert(key, value)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_cardano_keypair_bytes(
        &self,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(CARDANO_KEYPAIR)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }

    // --- Delegate Info ---

    pub fn set_delegate_pk(
        &self,
        network: &str,
        pk: &PublicKey,
    ) -> Result<(), StoreError> {
        let key = format!("{network}:pk");
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(DELEGATE_INFO)?;
            table.insert(key.as_str(), pk.0.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_delegate_pk(
        &self,
        network: &str,
    ) -> Result<Option<PublicKey>, StoreError> {
        let key = format!("{network}:pk");
        let txn = self.db.begin_read()?;
        let table = txn.open_table(DELEGATE_INFO)?;
        match table.get(key.as_str())? {
            Some(v) => {
                let bytes = v.value();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(bytes);
                Ok(Some(PublicKey(arr)))
            }
            None => Ok(None),
        }
    }

    pub fn set_script_address(
        &self,
        network: &str,
        addr: &str,
    ) -> Result<(), StoreError> {
        let key = format!("{network}:script_addr");
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(DELEGATE_INFO)?;
            table.insert(key.as_str(), addr.as_bytes())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_script_address(
        &self,
        network: &str,
    ) -> Result<Option<String>, StoreError> {
        let key = format!("{network}:script_addr");
        let txn = self.db.begin_read()?;
        let table = txn.open_table(DELEGATE_INFO)?;
        Ok(table
            .get(key.as_str())?
            .map(|v| String::from_utf8_lossy(v.value()).to_string()))
    }

    // --- Notes ---

    fn note_key(network: &str, nonce: &Hash) -> String {
        format!("{network}:{nonce}")
    }

    pub fn put_note(
        &self,
        network: &str,
        note: &Note,
        status: NoteStatus,
        created_at: u64,
    ) -> Result<(), StoreError> {
        let key = Self::note_key(network, &note.nonce);
        let stored = StoredNote {
            note: note.clone(),
            status,
            created_at,
        };
        let bytes = serde_json::to_vec(&stored)?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(NOTES)?;
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_note(
        &self,
        network: &str,
        nonce: &Hash,
    ) -> Result<Option<StoredNote>, StoreError> {
        let key = Self::note_key(network, nonce);
        let txn = self.db.begin_read()?;
        let table = txn.open_table(NOTES)?;
        match table.get(key.as_str())? {
            Some(v) => Ok(Some(serde_json::from_slice(v.value())?)),
            None => Ok(None),
        }
    }

    pub fn list_notes(
        &self,
        network: &str,
    ) -> Result<Vec<StoredNote>, StoreError> {
        let prefix = format!("{network}:");
        let txn = self.db.begin_read()?;
        let table = txn.open_table(NOTES)?;
        let mut notes = Vec::new();
        for entry in table.iter()? {
            let (k, v) = entry?;
            if k.value().starts_with(&prefix) {
                notes.push(serde_json::from_slice(v.value())?);
            }
        }
        Ok(notes)
    }

    pub fn update_note_status(
        &self,
        network: &str,
        nonce: &Hash,
        status: NoteStatus,
    ) -> Result<(), StoreError> {
        let key = Self::note_key(network, nonce);
        let txn = self.db.begin_read()?;
        let table = txn.open_table(NOTES)?;
        let stored: StoredNote = match table.get(key.as_str())? {
            Some(v) => serde_json::from_slice(v.value())?,
            None => return Err(StoreError::NotFound(key)),
        };
        drop(table);
        drop(txn);

        let updated = StoredNote { status, ..stored };
        let bytes = serde_json::to_vec(&updated)?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(NOTES)?;
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    // --- Activity ---

    pub fn put_activity(
        &self,
        network: &str,
        record: &ActivityRecord,
    ) -> Result<(), StoreError> {
        let key = format!("{network}:{}", record.id);
        let bytes = serde_json::to_vec(record)?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(ACTIVITY)?;
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn list_activity(
        &self,
        network: &str,
    ) -> Result<Vec<ActivityRecord>, StoreError> {
        let prefix = format!("{network}:");
        let txn = self.db.begin_read()?;
        let table = txn.open_table(ACTIVITY)?;
        let mut records = Vec::new();
        for entry in table.iter()? {
            let (k, v) = entry?;
            if k.value().starts_with(&prefix) {
                records.push(serde_json::from_slice(v.value())?);
            }
        }
        Ok(records)
    }

    // --- Blinding Factors ---

    pub fn put_blinding_factor(
        &self,
        network: &str,
        nonce: &Hash,
        factor: &[u8],
    ) -> Result<(), StoreError> {
        let key = format!("{network}:{nonce}");
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(BLINDING_FACTORS)?;
            table.insert(key.as_str(), factor)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_blinding_factor(
        &self,
        network: &str,
        nonce: &Hash,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let key = format!("{network}:{nonce}");
        let txn = self.db.begin_read()?;
        let table = txn.open_table(BLINDING_FACTORS)?;
        Ok(table.get(key.as_str())?.map(|v| v.value().to_vec()))
    }

    pub fn delete_blinding_factor(
        &self,
        network: &str,
        nonce: &Hash,
    ) -> Result<(), StoreError> {
        let key = format!("{network}:{nonce}");
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(BLINDING_FACTORS)?;
            table.remove(key.as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Atomically store a note and delete its blinding factor in one transaction.
    pub fn finalize_note(
        &self,
        network: &str,
        note: &Note,
        status: NoteStatus,
        created_at: u64,
    ) -> Result<(), StoreError> {
        let note_key = Self::note_key(network, &note.nonce);
        let bf_key = format!("{network}:{}", note.nonce);
        let stored = StoredNote {
            note: note.clone(),
            status,
            created_at,
        };
        let bytes = serde_json::to_vec(&stored)?;

        let txn = self.db.begin_write()?;
        {
            let mut notes_table = txn.open_table(NOTES)?;
            notes_table.insert(note_key.as_str(), bytes.as_slice())?;
            let mut bf_table = txn.open_table(BLINDING_FACTORS)?;
            bf_table.remove(bf_key.as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Scan for orphaned blinding factors whose nonce does not appear as an
    /// available note. These represent in-flight operations that crashed.
    pub fn scan_orphaned_blinding_factors(
        &self,
        network: &str,
    ) -> Result<Vec<OrphanedBlindingFactor>, StoreError> {
        let prefix = format!("{network}:");
        let txn = self.db.begin_read()?;
        let bf_table = txn.open_table(BLINDING_FACTORS)?;
        let notes_table = txn.open_table(NOTES)?;

        let mut orphans = Vec::new();
        for entry in bf_table.iter()? {
            let (k, v) = entry?;
            let key_str = k.value();
            if !key_str.starts_with(&prefix) {
                continue;
            }
            // Check if a matching note exists with available status
            let has_note = match notes_table.get(key_str)? {
                Some(note_bytes) => {
                    let stored: StoredNote =
                        serde_json::from_slice(note_bytes.value())?;
                    stored.status == NoteStatus::Available
                }
                None => false,
            };
            if !has_note {
                orphans.push(OrphanedBlindingFactor {
                    key: key_str.to_string(),
                    factor_bytes: v.value().to_vec(),
                });
            }
        }
        Ok(orphans)
    }

    // --- Offchain Requests ---

    pub fn put_offchain_request(
        &self,
        id: &str,
        data: &[u8],
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(OFFCHAIN_REQUESTS)?;
            table.insert(id, data)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_offchain_request(
        &self,
        id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(OFFCHAIN_REQUESTS)?;
        Ok(table.get(id)?.map(|v| v.value().to_vec()))
    }

    // --- Cardano UTxOs ---

    pub fn put_cardano_utxo(
        &self,
        key: &str,
        data: &[u8],
    ) -> Result<(), StoreError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(CARDANO_UTXOS)?;
            table.insert(key, data)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_cardano_utxo(
        &self,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(CARDANO_UTXOS)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::{AssetName, PolicyId, Signature};

    use super::*;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.redb");
        let store = Store::open(&path).unwrap();
        (dir, store)
    }

    fn test_note() -> Note {
        Note {
            amount: 1000,
            delegate: PublicKey([0x11; 32]),
            policy_id: PolicyId([0x22; 28]),
            asset_name: AssetName::new(b"USDM").unwrap(),
            nonce: Hash([0x33; 32]),
            signature: Signature([0x44; 32]),
            dleq: None,
        }
    }

    // --- Config tests ---

    #[test]
    fn config_roundtrip() {
        let (_dir, store) = temp_store();
        assert_eq!(store.get_config("label").unwrap(), None);
        store.set_config("label", "My Wallet").unwrap();
        assert_eq!(
            store.get_config("label").unwrap(),
            Some("My Wallet".to_string())
        );
    }

    #[test]
    fn config_last_network() {
        let (_dir, store) = temp_store();
        store.set_config("last_network", "preprod").unwrap();
        assert_eq!(
            store.get_config("last_network").unwrap(),
            Some("preprod".to_string())
        );
        store.set_config("last_network", "mainnet").unwrap();
        assert_eq!(
            store.get_config("last_network").unwrap(),
            Some("mainnet".to_string())
        );
    }

    // --- Provider Config tests ---

    #[test]
    fn provider_config_roundtrip() {
        let (_dir, store) = temp_store();
        store.set_provider_config("type", "blockfrost").unwrap();
        store.set_provider_config("api_key", "secret123").unwrap();
        assert_eq!(
            store.get_provider_config("type").unwrap(),
            Some("blockfrost".to_string())
        );
        assert_eq!(
            store.get_provider_config("api_key").unwrap(),
            Some("secret123".to_string())
        );
        assert_eq!(
            store.get_provider_config("base_url_override").unwrap(),
            None
        );
    }

    // --- Node Config tests ---

    #[test]
    fn node_url_per_network() {
        let (_dir, store) = temp_store();
        store.set_node_url("mainnet", "http://main:3000").unwrap();
        store
            .set_node_url("preprod", "http://preprod:3000")
            .unwrap();
        store
            .set_node_url("preview", "http://preview:3000")
            .unwrap();

        assert_eq!(
            store.get_node_url("mainnet").unwrap(),
            Some("http://main:3000".to_string())
        );
        assert_eq!(
            store.get_node_url("preprod").unwrap(),
            Some("http://preprod:3000".to_string())
        );
        assert_eq!(
            store.get_node_url("preview").unwrap(),
            Some("http://preview:3000".to_string())
        );
    }

    // --- Keypair tests ---

    #[test]
    fn keypair_bytes_roundtrip() {
        let (_dir, store) = temp_store();
        let sk = [0xABu8; 32];
        store.set_keypair_bytes("secret_key", &sk).unwrap();
        assert_eq!(
            store.get_keypair_bytes("secret_key").unwrap(),
            Some(sk.to_vec())
        );
        assert_eq!(store.get_keypair_bytes("ed25519_sk").unwrap(), None);
    }

    // --- Cardano Keypair tests ---

    #[test]
    fn cardano_keypair_roundtrip() {
        let (_dir, store) = temp_store();
        let sk = [0xCDu8; 32];
        let vk = [0xEFu8; 32];
        store.set_cardano_keypair_bytes("payment_sk", &sk).unwrap();
        store.set_cardano_keypair_bytes("payment_vk", &vk).unwrap();
        assert_eq!(
            store.get_cardano_keypair_bytes("payment_sk").unwrap(),
            Some(sk.to_vec())
        );
        assert_eq!(
            store.get_cardano_keypair_bytes("payment_vk").unwrap(),
            Some(vk.to_vec())
        );
    }

    // --- Delegate Info tests ---

    #[test]
    fn delegate_info_per_network() {
        let (_dir, store) = temp_store();
        let pk = PublicKey([0xAA; 32]);
        store.set_delegate_pk("preprod", &pk).unwrap();
        store
            .set_script_address("preprod", "addr_test1abc")
            .unwrap();

        assert_eq!(store.get_delegate_pk("preprod").unwrap(), Some(pk));
        assert_eq!(store.get_delegate_pk("mainnet").unwrap(), None);
        assert_eq!(
            store.get_script_address("preprod").unwrap(),
            Some("addr_test1abc".to_string())
        );
    }

    // --- Notes tests ---

    #[test]
    fn note_put_get_roundtrip() {
        let (_dir, store) = temp_store();
        let note = test_note();
        store
            .put_note("preprod", &note, NoteStatus::Available, 1000)
            .unwrap();

        let stored = store.get_note("preprod", &note.nonce).unwrap().unwrap();
        assert_eq!(stored.note, note);
        assert_eq!(stored.status, NoteStatus::Available);
        assert_eq!(stored.created_at, 1000);
    }

    #[test]
    fn note_list_filters_by_network() {
        let (_dir, store) = temp_store();
        let note = test_note();
        store
            .put_note("preprod", &note, NoteStatus::Available, 1000)
            .unwrap();

        let mut note2 = test_note();
        note2.nonce = Hash([0x55; 32]);
        store
            .put_note("mainnet", &note2, NoteStatus::Available, 2000)
            .unwrap();

        let preprod_notes = store.list_notes("preprod").unwrap();
        assert_eq!(preprod_notes.len(), 1);
        assert_eq!(preprod_notes[0].note.nonce, note.nonce);
    }

    #[test]
    fn note_status_update() {
        let (_dir, store) = temp_store();
        let note = test_note();
        store
            .put_note("preprod", &note, NoteStatus::Available, 1000)
            .unwrap();
        store
            .update_note_status("preprod", &note.nonce, NoteStatus::Spent)
            .unwrap();

        let stored = store.get_note("preprod", &note.nonce).unwrap().unwrap();
        assert_eq!(stored.status, NoteStatus::Spent);
    }

    // --- Activity tests ---

    #[test]
    fn activity_put_list() {
        let (_dir, store) = temp_store();
        let record = ActivityRecord {
            id: "act-1".to_string(),
            kind: "refresh".to_string(),
            timestamp: 1000,
            details: "split 1000 into 500+500".to_string(),
        };
        store.put_activity("preprod", &record).unwrap();
        let records = store.list_activity("preprod").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "act-1");
    }

    // --- Blinding Factors tests ---

    #[test]
    fn blinding_factor_lifecycle() {
        let (_dir, store) = temp_store();
        let nonce = Hash([0x99; 32]);
        let factor = [0xBBu8; 32];

        store
            .put_blinding_factor("preprod", &nonce, &factor)
            .unwrap();
        assert_eq!(
            store.get_blinding_factor("preprod", &nonce).unwrap(),
            Some(factor.to_vec())
        );

        store.delete_blinding_factor("preprod", &nonce).unwrap();
        assert_eq!(store.get_blinding_factor("preprod", &nonce).unwrap(), None);
    }

    #[test]
    fn finalize_note_is_atomic() {
        let (_dir, store) = temp_store();
        let note = test_note();
        let factor = [0xBBu8; 32];

        store
            .put_blinding_factor("preprod", &note.nonce, &factor)
            .unwrap();
        store
            .finalize_note("preprod", &note, NoteStatus::Available, 1000)
            .unwrap();

        assert!(store.get_note("preprod", &note.nonce).unwrap().is_some());
        assert_eq!(
            store.get_blinding_factor("preprod", &note.nonce).unwrap(),
            None
        );
    }

    #[test]
    fn orphaned_blinding_factors_detected() {
        let (_dir, store) = temp_store();
        let nonce = Hash([0x99; 32]);
        let factor = [0xBBu8; 32];

        // Blinding factor without a matching note = orphan
        store
            .put_blinding_factor("preprod", &nonce, &factor)
            .unwrap();

        let orphans = store.scan_orphaned_blinding_factors("preprod").unwrap();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].factor_bytes, factor.to_vec());
    }

    #[test]
    fn no_orphans_when_note_is_available() {
        let (_dir, store) = temp_store();
        let note = test_note();
        let factor = [0xBBu8; 32];

        store
            .put_blinding_factor("preprod", &note.nonce, &factor)
            .unwrap();
        store
            .put_note("preprod", &note, NoteStatus::Available, 1000)
            .unwrap();

        let orphans = store.scan_orphaned_blinding_factors("preprod").unwrap();
        assert_eq!(orphans.len(), 0);
    }

    #[test]
    fn spent_note_blinding_factor_is_orphan() {
        let (_dir, store) = temp_store();
        let note = test_note();
        let factor = [0xBBu8; 32];

        store
            .put_blinding_factor("preprod", &note.nonce, &factor)
            .unwrap();
        store
            .put_note("preprod", &note, NoteStatus::Spent, 1000)
            .unwrap();

        // Spent note + leftover blinding factor = orphan (the note isn't available)
        let orphans = store.scan_orphaned_blinding_factors("preprod").unwrap();
        assert_eq!(orphans.len(), 1);
    }

    // --- Offchain Requests tests ---

    #[test]
    fn offchain_request_roundtrip() {
        let (_dir, store) = temp_store();
        let data = b"request data";
        store.put_offchain_request("req-1", data).unwrap();
        assert_eq!(
            store.get_offchain_request("req-1").unwrap(),
            Some(data.to_vec())
        );
        assert_eq!(store.get_offchain_request("req-2").unwrap(), None);
    }

    // --- Cardano UTxOs tests ---

    #[test]
    fn cardano_utxo_roundtrip() {
        let (_dir, store) = temp_store();
        let data = b"utxo metadata";
        store.put_cardano_utxo("preprod:abc#0", data).unwrap();
        assert_eq!(
            store.get_cardano_utxo("preprod:abc#0").unwrap(),
            Some(data.to_vec())
        );
    }
}
