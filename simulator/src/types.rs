use std::collections::{HashMap, VecDeque};

use clap::Parser;
use mugraph_core::types::{Asset, BlindSignature, Note, PolicyId, PublicKey, Refresh};
use reqwest::Url;

#[derive(Debug, Parser)]
pub struct Args {
    /// Node base URL (e.g. http://127.0.0.1:9999)
    #[arg(long, default_value = "http://127.0.0.1:9999")]
    pub node_url: Url,
    /// Number of simulated wallets
    #[arg(long, default_value_t = 6)]
    pub wallets: usize,
    /// Number of distinct assets to simulate
    #[arg(long, default_value_t = 8)]
    pub assets: usize,
    /// Number of starting notes per wallet (per asset)
    #[arg(long, default_value_t = 2)]
    pub notes_per_wallet: usize,
    /// Minimum note/transfer amount
    #[arg(long, default_value_t = 1)]
    pub min_amount: u64,
    /// Maximum note/transfer amount
    #[arg(long, default_value_t = 50)]
    pub max_amount: u64,
    /// Milliseconds to wait between transaction attempts
    #[arg(long, default_value_t = 16)]
    pub tick_ms: u64,
    /// Maximum concurrent in-flight transactions
    #[arg(long, default_value_t = 16)]
    pub max_inflight: usize,
    /// RNG seed (optional) for reproducibility
    #[arg(long)]
    pub seed: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct Wallet {
    pub id: usize,
    pub notes: HashMap<Asset, Vec<Note>>,
    pub sent: u64,
    pub received: u64,
    pub failures: u64,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub wallets: Vec<Wallet>,
    pub assets: Vec<SimAsset>,
    pub delegate_pk: PublicKey,
    pub node_pk: Option<PublicKey>,
    pub logs: VecDeque<String>,
    pub inflight: usize,
    pub total_sent: u64,
    pub total_ok: u64,
    pub total_err: u64,
    pub last_failure: Option<String>,
    pub paused: bool,
    pub shutdown: bool,
}

impl AppState {
    pub fn log(&mut self, message: impl Into<String>) {
        let entry = message.into();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        self.logs.push_front(format!(
            "[{:>6}.{:03}] {}",
            now.as_secs(),
            now.subsec_millis(),
            entry
        ));
        if self.logs.len() > 200 {
            self.logs.pop_back();
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        let wallets = self
            .wallets
            .iter()
            .map(|wallet| WalletSnapshot {
                id: wallet.id,
                balances: self
                    .assets
                    .iter()
                    .map(|asset| {
                        let key = Asset {
                            policy_id: asset.policy_id,
                            asset_name: asset.asset_name,
                        };
                        let notes = wallet.notes.get(&key);
                        WalletBalance {
                            balance: notes
                                .map(|v| v.iter().map(|n| n.amount).sum::<u64>())
                                .unwrap_or(0),
                            notes: notes.map(|v| v.len()).unwrap_or(0),
                        }
                    })
                    .collect(),
                sent: wallet.sent,
                received: wallet.received,
                failures: wallet.failures,
            })
            .collect();

        AppSnapshot {
            wallets,
            assets: self.assets.clone(),
            delegate_pk: self.delegate_pk,
            node_pk: self.node_pk,
            logs: self.logs.clone(),
            inflight: self.inflight,
            total_sent: self.total_sent,
            total_ok: self.total_ok,
            total_err: self.total_err,
            last_failure: self.last_failure.clone(),
            paused: self.paused,
            shutdown: self.shutdown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub balance: u64,
    pub notes: usize,
}

#[derive(Debug, Clone)]
pub struct WalletSnapshot {
    pub id: usize,
    pub balances: Vec<WalletBalance>,
    pub sent: u64,
    pub received: u64,
    pub failures: u64,
}

#[derive(Debug, Clone)]
pub struct AppSnapshot {
    pub wallets: Vec<WalletSnapshot>,
    pub assets: Vec<SimAsset>,
    pub delegate_pk: PublicKey,
    pub node_pk: Option<PublicKey>,
    pub logs: VecDeque<String>,
    pub inflight: usize,
    pub total_sent: u64,
    pub total_ok: u64,
    pub total_err: u64,
    pub last_failure: Option<String>,
    pub paused: bool,
    pub shutdown: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SimCommand {
    TogglePause,
    Quit,
}

#[derive(Debug)]
pub struct PendingTx {
    pub id: u64,
    pub sender_id: usize,
    pub receiver_id: usize,
    pub asset: Asset,
    pub input_note: Note,
    pub spend_amount: u64,
    pub refresh: Refresh,
    pub owners: Vec<usize>,
}

#[derive(Debug)]
pub enum SimEvent {
    TxFinished {
        pending: PendingTx,
        result: std::result::Result<Vec<BlindSignature>, String>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct SimAsset {
    pub policy_id: PolicyId,
    pub asset_name: mugraph_core::types::AssetName,
    pub name: &'static str,
    pub policy_id_hex: &'static str,
}
