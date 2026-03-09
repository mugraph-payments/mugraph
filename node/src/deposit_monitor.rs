use mugraph_core::{
    error::Error,
    types::{DepositRecord, UtxoRef},
};
use redb::ReadableTable;

use crate::{
    database::{DEPOSITS, Database},
    provider::{Provider, UtxoInfo},
};

/// Configuration for deposit monitoring
#[derive(Debug, Clone)]
pub struct DepositMonitorConfig {
    /// Number of blocks for confirmation depth (default: 15)
    pub confirm_depth: u64,
    /// Number of blocks after which unclaimed deposits expire (default: 1440 = ~24 hours)
    pub expiration_blocks: u64,
    /// Minimum deposit value in lovelace
    pub min_deposit_value: u64,
    /// Revalidation interval in seconds
    pub revalidation_interval: u64,
}

impl Default for DepositMonitorConfig {
    fn default() -> Self {
        Self {
            confirm_depth: 15,
            expiration_blocks: 1440,
            min_deposit_value: 1_000_000, // 1 ADA
            revalidation_interval: 60,    // 1 minute
        }
    }
}

/// Deposit monitor for handling reorgs and expirations
pub struct DepositMonitor {
    config: DepositMonitorConfig,
    database: std::sync::Arc<Database>,
    provider: Provider,
}

impl DepositMonitor {
    pub fn new(
        config: DepositMonitorConfig,
        database: std::sync::Arc<Database>,
        provider: Provider,
    ) -> Self {
        Self {
            config,
            database,
            provider,
        }
    }

    /// Start the deposit monitoring background task
    pub async fn start(self) {
        tracing::info!(
            "Starting deposit monitor with {} block confirmation depth",
            self.config.confirm_depth
        );

        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.revalidation_interval),
        );

        loop {
            interval.tick().await;

            if let Err(e) = self.check_deposits().await {
                tracing::error!("Error checking deposits: {}", e);
            }
        }
    }

    /// Check all pending deposits for:
    /// 1. Reorg - if UTxO no longer exists on chain
    /// 2. Expiration - if deposit is older than expiration_blocks
    /// 3. Spent - if UTxO was spent (should update status)
    async fn check_deposits(&self) -> Result<(), Error> {
        let tip =
            self.provider
                .get_tip()
                .await
                .map_err(|e| Error::NetworkError {
                    reason: format!("Failed to get chain tip: {}", e),
                })?;

        tracing::debug!(
            "Deposit monitor running at block {} (checking pending deposits)",
            tip.block_height
        );

        // Get current timestamp
        let now = secs_since_unix_epoch(std::time::SystemTime::now());

        // Scan all deposits and check pending ones
        // Note: This scans the entire table. For large deployments, consider:
        // 1. A separate index table for pending deposits only
        // 2. Range queries if implementing a time-based key structure
        let pending_deposits = self.get_pending_deposits()?;

        // Load script address once for this pass
        let script_address = self.load_script_address()?;

        tracing::info!(
            "Found {} pending deposits to check",
            pending_deposits.len()
        );

        for (utxo_ref, record) in pending_deposits {
            // Check expiration first (both timestamp and block horizon)
            let expired_by_time = now > record.expires_at && !record.spent;
            let expired_by_blocks = is_expired_by_blocks(
                tip.block_height,
                record.block_height,
                self.config.expiration_blocks,
            ) && !record.spent;

            if expired_by_time || expired_by_blocks {
                tracing::info!(
                    "Deposit {} expired (expired_at {}, now {}, blocks elapsed {})",
                    hex::encode(&utxo_ref.tx_hash[..8]),
                    record.expires_at,
                    now,
                    tip.block_height.saturating_sub(record.block_height)
                );

                // Mark as expired by setting spent flag
                // This prevents the deposit from being claimed
                self.mark_deposit_spent(&utxo_ref)?;
                continue;
            }

            // Skip if already confirmed (we only re-check young deposits)
            let blocks_elapsed =
                tip.block_height.saturating_sub(record.block_height);
            if blocks_elapsed >= self.config.confirm_depth {
                // Deposit is confirmed, no need to re-check
                continue;
            }

            // Re-validate UTxO exists on chain (reorg check)
            match self
                .validate_utxo_on_chain(&utxo_ref, script_address.as_deref())
                .await
            {
                Ok(true) => {
                    // UTxO still exists, deposit is valid
                    tracing::debug!(
                        "Deposit {} still valid at block {}",
                        hex::encode(&utxo_ref.tx_hash[..8]),
                        record.block_height
                    );
                }
                Ok(false) => {
                    // UTxO no longer exists - was spent or reorged
                    tracing::warn!(
                        "Deposit {} no longer exists on chain (possible reorg or spend)",
                        hex::encode(&utxo_ref.tx_hash[..8])
                    );

                    // Mark as spent/invalid
                    self.mark_deposit_spent(&utxo_ref)?;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to validate deposit {}: {}",
                        hex::encode(&utxo_ref.tx_hash[..8]),
                        e
                    );
                    // Don't mark as spent on error, will retry next interval
                }
            }
        }

        Ok(())
    }

    /// Get all pending (unspent) deposits from the database
    fn get_pending_deposits(
        &self,
    ) -> Result<Vec<(UtxoRef, DepositRecord)>, Error> {
        let read_tx = self.database.read()?;
        let table = read_tx.open_table(DEPOSITS)?;

        let mut pending = Vec::new();

        // Iterate over all deposits
        // Note: redb's iterator is efficient for small-to-medium datasets
        for item in table.iter()? {
            let (k, v) = item?;
            let utxo_ref = k.value();
            let record = v.value();

            if !record.spent {
                pending.push((utxo_ref, record));
            }
        }

        Ok(pending)
    }

    /// Validate that a UTxO still exists on chain
    async fn validate_utxo_on_chain(
        &self,
        utxo_ref: &UtxoRef,
        script_address: Option<&str>,
    ) -> Result<bool, Error> {
        let tx_hash = hex::encode(utxo_ref.tx_hash);

        match self.provider.get_utxo(&tx_hash, utxo_ref.index).await {
            Ok(Some(utxo_info)) => {
                // UTxO exists - verify it still has assets (not emptied)
                // and is still at our script address (reorg protection)
                if let Some(expected_addr) = script_address
                    && utxo_info.address != expected_addr
                {
                    tracing::warn!(
                        "UTxO {} moved from script address (was {}, now {})",
                        tx_hash,
                        expected_addr,
                        utxo_info.address
                    );
                    return Ok(false);
                }

                if !utxo_meets_min_deposit(
                    &utxo_info,
                    self.config.min_deposit_value,
                ) {
                    tracing::warn!(
                        "UTxO {} below min deposit value {}",
                        tx_hash,
                        self.config.min_deposit_value
                    );
                    return Ok(false);
                }

                Ok(!utxo_info.amount.is_empty())
            }
            Ok(None) => {
                // UTxO no longer exists - was spent or reorged
                Ok(false)
            }
            Err(e) => {
                tracing::error!("Failed to fetch UTxO {}: {}", tx_hash, e);
                Err(Error::NetworkError {
                    reason: format!("Failed to validate UTxO: {}", e),
                })
            }
        }
    }

    /// Load current script address from CARDANO_WALLET, if present
    fn load_script_address(&self) -> Result<Option<String>, Error> {
        use crate::database::CARDANO_WALLET;

        let read_tx = self.database.read()?;
        let table = read_tx.open_table(CARDANO_WALLET)?;
        Ok(table.get("wallet")?.map(|w| w.value().script_address))
    }

    /// Mark a deposit as spent (called when withdrawal is processed)
    pub fn mark_deposit_spent(&self, utxo_ref: &UtxoRef) -> Result<(), Error> {
        // First, read the existing record
        let existing_record = {
            let read_tx = self.database.read()?;
            let table = read_tx.open_table(DEPOSITS)?;
            table.get(utxo_ref)?.map(|v| v.value())
        };

        // Then update it if found
        if let Some(mut record) = existing_record {
            let write_tx = self.database.write()?;
            {
                let mut table = write_tx.open_table(DEPOSITS)?;
                record.spent = true;
                table.insert(utxo_ref, &record)?;
            }
            write_tx.commit()?;
        }

        Ok(())
    }

    /// Check if a deposit is valid (exists, confirmed, not expired, not spent)
    pub async fn is_deposit_valid(
        &self,
        utxo_ref: &UtxoRef,
    ) -> Result<bool, Error> {
        let read_tx = self.database.read()?;
        let table = read_tx.open_table(DEPOSITS)?;

        match table.get(utxo_ref)? {
            Some(record) => {
                let record = record.value();

                // Check if spent
                if record.spent {
                    return Ok(false);
                }

                // Check if expired
                let now = secs_since_unix_epoch(std::time::SystemTime::now());

                if now > record.expires_at {
                    return Ok(false);
                }

                // Check confirmation depth
                let tip = self.provider.get_tip().await.map_err(|e| {
                    Error::NetworkError {
                        reason: format!("Failed to get chain tip: {}", e),
                    }
                })?;

                let blocks_elapsed =
                    tip.block_height.saturating_sub(record.block_height);
                if blocks_elapsed < self.config.confirm_depth {
                    return Ok(false); // Not yet confirmed
                }

                Ok(true)
            }
            None => Ok(false),
        }
    }
}

fn secs_since_unix_epoch(now: std::time::SystemTime) -> u64 {
    now.duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_expired_by_blocks(
    current_height: u64,
    deposit_height: u64,
    expiration_blocks: u64,
) -> bool {
    current_height.saturating_sub(deposit_height) > expiration_blocks
}

fn utxo_meets_min_deposit(
    utxo_info: &UtxoInfo,
    min_deposit_value: u64,
) -> bool {
    let lovelace = utxo_info
        .amount
        .iter()
        .find(|a| a.unit == "lovelace")
        .and_then(|a| a.quantity.parse::<u64>().ok())
        .unwrap_or(0);

    lovelace >= min_deposit_value
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
        routing::get,
    };
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::*;
    use crate::{database::CARDANO_WALLET, provider::AssetAmount};

    #[derive(Clone)]
    struct MockState {
        utxo_status: Arc<Mutex<StatusCode>>,
        utxo_address: String,
    }

    async fn latest_block() -> impl IntoResponse {
        (
            StatusCode::OK,
            axum::Json(json!({"slot": 1000, "hash": "h", "height": 100})),
        )
    }

    async fn tx_info() -> impl IntoResponse {
        (StatusCode::OK, axum::Json(json!({"block_height": 90})))
    }

    async fn tx_utxos(
        State(state): State<MockState>,
        Path(_tx_hash): Path<String>,
    ) -> impl IntoResponse {
        let status = *state.utxo_status.lock().await;
        match status {
            StatusCode::OK => (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": "ab",
                    "outputs": [{
                        "output_index": 0,
                        "address": state.utxo_address,
                        "amount": [{"unit":"lovelace","quantity":"1000000"}],
                        "data_hash": null,
                        "reference_script_hash": null
                    }]
                })),
            )
                .into_response(),
            StatusCode::NOT_FOUND => {
                (StatusCode::NOT_FOUND, "not found").into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "boom").into_response(),
        }
    }

    async fn spawn_mock_server(
        utxo_status: StatusCode,
        utxo_address: &str,
    ) -> String {
        let state = MockState {
            utxo_status: Arc::new(Mutex::new(utxo_status)),
            utxo_address: utxo_address.to_string(),
        };

        let app = Router::new()
            .route("/blocks/latest", get(latest_block))
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .with_state(state);

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    fn temp_db() -> Arc<Database> {
        let path = std::env::temp_dir().join(format!(
            "mugraph-deposit-monitor-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = Arc::new(Database::setup(path).unwrap());
        db.migrate().unwrap();
        db
    }

    fn seed_wallet_and_deposit(
        db: &Arc<Database>,
        utxo_ref: UtxoRef,
        record: DepositRecord,
    ) {
        let w = db.write().unwrap();
        {
            let mut wallet = w.open_table(CARDANO_WALLET).unwrap();
            wallet
                .insert(
                    "wallet",
                    &mugraph_core::types::CardanoWallet::new(
                        vec![1u8; 32],
                        vec![2u8; 32],
                        vec![],
                        vec![],
                        "addr_test1script".to_string(),
                        "preprod".to_string(),
                    ),
                )
                .unwrap();

            let mut deposits = w.open_table(DEPOSITS).unwrap();
            deposits.insert(&utxo_ref, &record).unwrap();
        }
        w.commit().unwrap();
    }

    #[test]
    fn test_default_config() {
        let config = DepositMonitorConfig::default();
        assert_eq!(config.confirm_depth, 15);
        assert_eq!(config.expiration_blocks, 1440);
        assert_eq!(config.min_deposit_value, 1_000_000);
    }

    #[test]
    fn expiration_by_blocks_uses_configured_horizon() {
        assert!(!is_expired_by_blocks(200, 100, 100));
        assert!(is_expired_by_blocks(201, 100, 100));
    }

    #[test]
    fn min_deposit_value_is_enforced_from_config() {
        let utxo = UtxoInfo {
            tx_hash: "ab".repeat(32),
            output_index: 0,
            address: "addr_test1...".to_string(),
            amount: vec![AssetAmount {
                unit: "lovelace".to_string(),
                quantity: "999999".to_string(),
            }],
            datum_hash: None,
            datum: None,
            script_ref: None,
            block_height: Some(10),
        };

        assert!(!utxo_meets_min_deposit(&utxo, 1_000_000));
        assert!(utxo_meets_min_deposit(&utxo, 999_999));
    }

    #[test]
    fn secs_since_unix_epoch_clamps_pre_epoch_to_zero() {
        let pre_epoch = std::time::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(1))
            .expect("pre epoch time");
        assert_eq!(secs_since_unix_epoch(pre_epoch), 0);
    }

    #[tokio::test]
    async fn check_deposits_marks_missing_utxo_as_spent() {
        let base_url =
            spawn_mock_server(StatusCode::NOT_FOUND, "addr_test1script").await;
        let provider = Provider::new(
            "blockfrost",
            "key".to_string(),
            "preprod".to_string(),
            Some(base_url),
        )
        .unwrap();
        let db = temp_db();

        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        let now = secs_since_unix_epoch(std::time::SystemTime::now());
        let record = DepositRecord::new(99, now, now + 3600);
        seed_wallet_and_deposit(&db, utxo_ref.clone(), record);

        let monitor = DepositMonitor::new(
            DepositMonitorConfig {
                confirm_depth: 1000,
                expiration_blocks: 10_000,
                min_deposit_value: 1_000_000,
                revalidation_interval: 60,
            },
            db.clone(),
            provider,
        );

        monitor.check_deposits().await.unwrap();

        let r = db.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let stored = deposits.get(&utxo_ref).unwrap().unwrap().value();
        assert!(stored.spent);
    }

    #[tokio::test]
    async fn check_deposits_keeps_unspent_on_transient_provider_error() {
        let base_url = spawn_mock_server(
            StatusCode::INTERNAL_SERVER_ERROR,
            "addr_test1script",
        )
        .await;
        let provider = Provider::new(
            "blockfrost",
            "key".to_string(),
            "preprod".to_string(),
            Some(base_url),
        )
        .unwrap();
        let db = temp_db();

        let utxo_ref = UtxoRef::new([0xcdu8; 32], 0);
        let now = secs_since_unix_epoch(std::time::SystemTime::now());
        let record = DepositRecord::new(99, now, now + 3600);
        seed_wallet_and_deposit(&db, utxo_ref.clone(), record);

        let monitor = DepositMonitor::new(
            DepositMonitorConfig {
                confirm_depth: 1000,
                expiration_blocks: 10_000,
                min_deposit_value: 1_000_000,
                revalidation_interval: 60,
            },
            db.clone(),
            provider,
        );

        monitor.check_deposits().await.unwrap();

        let r = db.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let stored = deposits.get(&utxo_ref).unwrap().unwrap().value();
        assert!(!stored.spent);
    }

    #[tokio::test]
    async fn check_deposits_marks_time_expired_as_spent_without_chain_lookup() {
        let base_url =
            spawn_mock_server(StatusCode::OK, "addr_test1script").await;
        let provider = Provider::new(
            "blockfrost",
            "key".to_string(),
            "preprod".to_string(),
            Some(base_url),
        )
        .unwrap();
        let db = temp_db();

        let utxo_ref = UtxoRef::new([0xeeu8; 32], 0);
        let now = secs_since_unix_epoch(std::time::SystemTime::now());
        let record = DepositRecord::new(
            99,
            now.saturating_sub(3600),
            now.saturating_sub(1),
        );
        seed_wallet_and_deposit(&db, utxo_ref.clone(), record);

        let monitor = DepositMonitor::new(
            DepositMonitorConfig::default(),
            db.clone(),
            provider,
        );
        monitor.check_deposits().await.unwrap();

        let r = db.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let stored = deposits.get(&utxo_ref).unwrap().unwrap().value();
        assert!(stored.spent);
    }

    #[tokio::test]
    async fn check_deposits_marks_block_horizon_expired_as_spent() {
        let base_url =
            spawn_mock_server(StatusCode::OK, "addr_test1script").await;
        let provider = Provider::new(
            "blockfrost",
            "key".to_string(),
            "preprod".to_string(),
            Some(base_url),
        )
        .unwrap();
        let db = temp_db();

        let utxo_ref = UtxoRef::new([0xefu8; 32], 0);
        let now = secs_since_unix_epoch(std::time::SystemTime::now());
        // tip=100 from mock, with expiration_blocks=5 => record at 90 is expired by blocks.
        let record = DepositRecord::new(90, now, now + 3600);
        seed_wallet_and_deposit(&db, utxo_ref.clone(), record);

        let monitor = DepositMonitor::new(
            DepositMonitorConfig {
                confirm_depth: 1000,
                expiration_blocks: 5,
                min_deposit_value: 1_000_000,
                revalidation_interval: 60,
            },
            db.clone(),
            provider,
        );

        monitor.check_deposits().await.unwrap();

        let r = db.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let stored = deposits.get(&utxo_ref).unwrap().unwrap().value();
        assert!(stored.spent);
    }

    #[tokio::test]
    async fn check_deposits_marks_script_address_drift_as_spent() {
        let base_url =
            spawn_mock_server(StatusCode::OK, "addr_test1different").await;
        let provider = Provider::new(
            "blockfrost",
            "key".to_string(),
            "preprod".to_string(),
            Some(base_url),
        )
        .unwrap();
        let db = temp_db();

        let utxo_ref = UtxoRef::new([0xf0u8; 32], 0);
        let now = secs_since_unix_epoch(std::time::SystemTime::now());
        let record = DepositRecord::new(99, now, now + 3600);
        seed_wallet_and_deposit(&db, utxo_ref.clone(), record);

        let monitor = DepositMonitor::new(
            DepositMonitorConfig {
                confirm_depth: 1000,
                expiration_blocks: 10_000,
                min_deposit_value: 1_000_000,
                revalidation_interval: 60,
            },
            db.clone(),
            provider,
        );

        monitor.check_deposits().await.unwrap();

        let r = db.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let stored = deposits.get(&utxo_ref).unwrap().unwrap().value();
        assert!(stored.spent);
    }
}
