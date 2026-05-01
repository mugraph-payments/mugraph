use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::tx::ParsedTx;

/// 32-byte transaction hash, hex-encoded for stable map keys.
pub type TxHash = String;
/// 32-byte datum hash, hex-encoded.
pub type DatumHash = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoEntry {
    pub tx_hash: TxHash,
    pub output_index: u16,
    pub address: String,
    pub lovelace: u64,
    pub datum_hash: Option<DatumHash>,
    pub inline_datum_cbor: Option<String>,
    /// Block this UTxO was created in. `None` while sitting in the
    /// pending-block buffer.
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRecord {
    pub tx_hash: TxHash,
    pub inputs: Vec<(TxHash, u16)>,
    pub outputs: Vec<UtxoEntry>,
    pub fee: u64,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub slot: u64,
    pub hash: String,
    pub tx_hashes: Vec<TxHash>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MineMode {
    /// Auto-mine a block immediately after each accepted submit.
    OnSubmit,
    /// Only mine when `mine` is invoked explicitly (or via the admin endpoint).
    Manual,
}

#[derive(Debug)]
pub struct Chain {
    blocks: Vec<Block>,
    utxos: BTreeMap<(TxHash, u16), UtxoEntry>,
    txs: HashMap<TxHash, TxRecord>,
    datums: HashMap<DatumHash, String>,
    pending: Vec<TxRecord>,
    mode: MineMode,
}

#[derive(Debug)]
pub enum SubmitError {
    Decode(String),
    InputMissing { tx_hash: TxHash, index: u16 },
    DuplicateTx(TxHash),
}

impl std::fmt::Display for SubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "tx decode failed: {e}"),
            Self::InputMissing { tx_hash, index } => {
                write!(f, "input not found: {tx_hash}#{index}")
            }
            Self::DuplicateTx(h) => write!(f, "duplicate tx hash: {h}"),
        }
    }
}

impl std::error::Error for SubmitError {}

impl Chain {
    pub fn new(mode: MineMode) -> Self {
        let genesis = Block {
            height: 0,
            slot: 0,
            hash: "00".repeat(32),
            tx_hashes: Vec::new(),
        };
        Self {
            blocks: vec![genesis],
            utxos: BTreeMap::new(),
            txs: HashMap::new(),
            datums: HashMap::new(),
            pending: Vec::new(),
            mode,
        }
    }

    pub fn mode(&self) -> MineMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: MineMode) {
        self.mode = mode;
    }

    pub fn tip(&self) -> &Block {
        self.blocks
            .last()
            .expect("blocks is non-empty post-construction")
    }

    pub fn block_at(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    /// Mint a UTxO out of band. Used by the admin faucet endpoint to fund a
    /// wallet's external address before any chain activity.
    ///
    /// The faucet UTxO is wrapped in a synthetic tx whose hash is derived
    /// from a counter so it parses through the same code path as a real one.
    pub fn faucet(&mut self, address: &str, lovelace: u64) -> UtxoEntry {
        let counter = self.txs.len() as u64 + self.pending.len() as u64;
        let mut tx_hash_bytes = [0u8; 32];
        tx_hash_bytes[0] = 0xFA;
        tx_hash_bytes[1..9].copy_from_slice(&counter.to_be_bytes());
        let tx_hash = hex::encode(tx_hash_bytes);

        let utxo = UtxoEntry {
            tx_hash: tx_hash.clone(),
            output_index: 0,
            address: address.to_string(),
            lovelace,
            datum_hash: None,
            inline_datum_cbor: None,
            block_height: None,
        };
        self.pending.push(TxRecord {
            tx_hash,
            inputs: Vec::new(),
            outputs: vec![utxo.clone()],
            fee: 0,
            block_height: None,
        });

        if self.mode == MineMode::OnSubmit {
            self.mine(1);
        }
        utxo
    }

    /// Apply a submitted Cardano transaction. Consumes inputs from the live
    /// UTxO set, mints outputs, and stages the resulting record into the
    /// pending block. Returns the tx hash.
    pub fn submit(&mut self, tx_cbor: &[u8]) -> Result<TxHash, SubmitError> {
        let parsed =
            ParsedTx::from_cbor(tx_cbor).map_err(SubmitError::Decode)?;

        if self.txs.contains_key(&parsed.tx_hash)
            || self.pending.iter().any(|r| r.tx_hash == parsed.tx_hash)
        {
            return Err(SubmitError::DuplicateTx(parsed.tx_hash));
        }

        for (h, i) in &parsed.inputs {
            if !self.utxos.contains_key(&(h.clone(), *i)) {
                return Err(SubmitError::InputMissing {
                    tx_hash: h.clone(),
                    index: *i,
                });
            }
        }

        // Consume inputs (only after we've confirmed all exist, so a partial
        // tx never half-applies).
        for (h, i) in &parsed.inputs {
            self.utxos.remove(&(h.clone(), *i));
        }

        let mut outputs = Vec::with_capacity(parsed.outputs.len());
        for (idx, out) in parsed.outputs.iter().enumerate() {
            let entry = UtxoEntry {
                tx_hash: parsed.tx_hash.clone(),
                output_index: idx as u16,
                address: out.address.clone(),
                lovelace: out.lovelace,
                datum_hash: out.datum_hash.clone(),
                inline_datum_cbor: out.inline_datum_cbor.clone(),
                block_height: None,
            };
            outputs.push(entry);
            if let (Some(h), Some(cbor)) =
                (out.datum_hash.clone(), out.inline_datum_cbor.clone())
            {
                self.datums.insert(h, cbor);
            }
        }

        let record = TxRecord {
            tx_hash: parsed.tx_hash.clone(),
            inputs: parsed.inputs.clone(),
            outputs,
            fee: parsed.fee,
            block_height: None,
        };
        self.pending.push(record);

        if self.mode == MineMode::OnSubmit {
            self.mine(1);
        }
        Ok(parsed.tx_hash)
    }

    /// Mine `count` blocks. The first new block consumes the pending buffer;
    /// subsequent blocks are empty.
    pub fn mine(&mut self, count: u64) -> u64 {
        let mut minted = 0;
        for _ in 0..count {
            let height = self.tip().height + 1;
            let slot = height * 20;
            let mut tx_hashes = Vec::new();

            let pending = std::mem::take(&mut self.pending);
            for mut record in pending {
                record.block_height = Some(height);
                for out in &mut record.outputs {
                    out.block_height = Some(height);
                    self.utxos.insert(
                        (out.tx_hash.clone(), out.output_index),
                        out.clone(),
                    );
                }
                tx_hashes.push(record.tx_hash.clone());
                self.txs.insert(record.tx_hash.clone(), record);
            }

            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = 0xB1;
            hash_bytes[1..9].copy_from_slice(&height.to_be_bytes());
            let hash = hex::encode(hash_bytes);

            self.blocks.push(Block {
                height,
                slot,
                hash,
                tx_hashes,
            });
            minted += 1;
        }
        minted
    }

    pub fn utxos_at(&self, address: &str) -> Vec<UtxoEntry> {
        self.utxos
            .values()
            .filter(|u| u.address == address)
            .cloned()
            .collect()
    }

    pub fn tx(&self, tx_hash: &str) -> Option<&TxRecord> {
        self.txs.get(tx_hash)
    }

    pub fn datum(&self, datum_hash: &str) -> Option<&String> {
        self.datums.get(datum_hash)
    }

    pub fn live_utxo_count(&self) -> usize {
        self.utxos.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn tx_count(&self) -> usize {
        self.txs.len()
    }

    pub fn reset(&mut self) {
        let mode = self.mode;
        *self = Self::new(mode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faucet_then_mine_creates_live_utxo() {
        let mut chain = Chain::new(MineMode::Manual);
        let utxo = chain.faucet("addr_test1abc", 100_000_000);
        assert_eq!(chain.pending_count(), 1);
        assert_eq!(chain.live_utxo_count(), 0);

        chain.mine(1);
        assert_eq!(chain.pending_count(), 0);
        assert_eq!(chain.live_utxo_count(), 1);
        let live = chain.utxos_at("addr_test1abc");
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].lovelace, 100_000_000);
        assert_eq!(live[0].block_height, Some(1));
        assert_eq!(live[0].tx_hash, utxo.tx_hash);
    }

    #[test]
    fn faucet_auto_mines_when_mode_on_submit() {
        let mut chain = Chain::new(MineMode::OnSubmit);
        chain.faucet("addr_test1abc", 5);
        assert_eq!(chain.live_utxo_count(), 1);
        assert_eq!(chain.tip().height, 1);
    }

    #[test]
    fn tip_increments_per_mine() {
        let mut chain = Chain::new(MineMode::Manual);
        assert_eq!(chain.tip().height, 0);
        chain.mine(3);
        assert_eq!(chain.tip().height, 3);
    }

    #[test]
    fn reset_wipes_state_but_keeps_mode() {
        let mut chain = Chain::new(MineMode::OnSubmit);
        chain.faucet("addr1", 10);
        chain.mine(2);
        assert!(chain.tip().height > 0);

        chain.reset();
        assert_eq!(chain.mode(), MineMode::OnSubmit);
        assert_eq!(chain.tip().height, 0);
        assert_eq!(chain.live_utxo_count(), 0);
        assert_eq!(chain.tx_count(), 0);
    }
}
