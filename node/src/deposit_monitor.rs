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

        // TODO: Implement deposit monitoring
        // redb doesn't support full table iteration efficiently
        // We need to either:
        // 1. Use a secondary index for pending deposits
        // 2. Store pending deposits in a separate table
        // 3. Accept that we'll scan the entire table

        // For now, just log that monitoring is running
        tracing::debug!(
            "Deposit monitor running at block {} (checking pending deposits)",
            tip.block_height
        );

        Ok(())
    }

    /// Validate that a UTxO still exists on chain
    async fn validate_utxo_on_chain(
        &self,
        utxo_ref: &UtxoRef,
        record: &DepositRecord,
    ) -> Result<bool, Error> {
        let tx_hash = hex::encode(utxo_ref.tx_hash);

        match self.provider.get_utxo(&tx_hash, utxo_ref.index).await {
            Ok(Some(utxo_info)) => {
                // UTxO exists - check if it's still at our script
                // TODO: Compare with expected script address from wallet
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
