use color_eyre::eyre::Result;
use mugraph_core::error::Error;
use whisky_csl::csl;

use crate::tx_signer::compute_tx_hash;

pub(crate) struct ParsedWithdrawalTx {
    pub(super) tx_cbor: Vec<u8>,
    pub(super) tx: csl::Transaction,
    pub(super) tx_hash: [u8; 32],
    pub(super) tx_hash_hex: String,
}

impl ParsedWithdrawalTx {
    pub(super) fn parse(tx_cbor_hex: &str) -> Result<Self, Error> {
        let tx_cbor =
            hex::decode(tx_cbor_hex).map_err(|e| Error::InvalidInput {
                reason: format!("Invalid tx_cbor hex: {}", e),
            })?;
        let tx =
            csl::Transaction::from_bytes(tx_cbor.clone()).map_err(|e| {
                Error::InvalidInput {
                    reason: format!("Invalid transaction CBOR: {}", e),
                }
            })?;
        let tx_hash =
            compute_tx_hash(&tx_cbor).map_err(|e| Error::InvalidInput {
                reason: format!("Failed to compute tx hash: {}", e),
            })?;

        Ok(Self {
            tx_cbor,
            tx,
            tx_hash_hex: hex::encode(tx_hash),
            tx_hash,
        })
    }
}
