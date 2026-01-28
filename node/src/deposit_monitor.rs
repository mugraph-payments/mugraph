use mugraph_core::{
    error::Error,
    types::{DepositRecord, UtxoRef},
};
use redb::ReadableTable;

use crate::{
    database::{DEPOSITS, Database},
    provider::Provider,
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

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.revalidation_interval,
        ));

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
        let tip = self
            .provider
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Scan all deposits and check pending ones
        // Note: This scans the entire table. For large deployments, consider:
        // 1. A separate index table for pending deposits only
        // 2. Range queries if implementing a time-based key structure
        let pending_deposits = self.get_pending_deposits()?;

        // Load script address once for this pass
        let script_address = self.load_script_address()?;

        tracing::info!("Found {} pending deposits to check", pending_deposits.len());

        for (utxo_ref, record) in pending_deposits {
            // Check expiration first
            if now > record.expires_at && !record.spent {
                tracing::info!(
                    "Deposit {} expired (expired at {}, now {})",
                    hex::encode(&utxo_ref.tx_hash[..8]),
                    record.expires_at,
                    now
                );

                // Mark as expired by setting spent flag
                // This prevents the deposit from being claimed
                self.mark_deposit_spent(&utxo_ref)?;
                continue;
            }

            // Skip if already confirmed (we only re-check young deposits)
            let blocks_elapsed = tip.block_height.saturating_sub(record.block_height);
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
    fn get_pending_deposits(&self) -> Result<Vec<(UtxoRef, DepositRecord)>, Error> {
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
                if let Some(expected_addr) = script_address {
                    if utxo_info.address != expected_addr {
                        tracing::warn!(
                            "UTxO {} moved from script address (was {}, now {})",
                            tx_hash,
                            expected_addr,
                            utxo_info.address
                        );
                        return Ok(false);
                    }
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
    pub async fn is_deposit_valid(&self, utxo_ref: &UtxoRef) -> Result<bool, Error> {
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
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > record.expires_at {
                    return Ok(false);
                }

                // Check confirmation depth
                let tip = self
                    .provider
                    .get_tip()
                    .await
                    .map_err(|e| Error::NetworkError {
                        reason: format!("Failed to get chain tip: {}", e),
                    })?;

                let blocks_elapsed = tip.block_height.saturating_sub(record.block_height);
                if blocks_elapsed < self.config.confirm_depth {
                    return Ok(false); // Not yet confirmed
                }

                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DepositMonitorConfig::default();
        assert_eq!(config.confirm_depth, 15);
        assert_eq!(config.expiration_blocks, 1440);
        assert_eq!(config.min_deposit_value, 1_000_000);
    }
}
