use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

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

    pub fn snapshot(
        &self,
        conservation_checks: u64,
        max_inflight: usize,
        tx_per_sec: f64,
        success_rate: f64,
    ) -> AppSnapshot {
        let wallets: Vec<WalletSnapshot> = self
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

        let asset_summaries = self
            .assets
            .iter()
            .map(|sim_asset| {
                let key = Asset {
                    policy_id: sim_asset.policy_id,
                    asset_name: sim_asset.asset_name,
                };
                let mut total_supply: u64 = 0;
                let mut total_notes: usize = 0;
                let mut wallets_holding: usize = 0;
                for wallet in &self.wallets {
                    if let Some(notes) = wallet.notes.get(&key)
                        && !notes.is_empty()
                    {
                        wallets_holding += 1;
                        total_notes += notes.len();
                        total_supply += notes.iter().map(|n| n.amount).sum::<u64>();
                    }
                }
                AssetSummary {
                    name: sim_asset.name,
                    policy_id_hex: sim_asset.policy_id_hex,
                    total_supply,
                    total_notes,
                    wallets_holding,
                }
            })
            .collect();

        AppSnapshot {
            wallets,
            asset_summaries,
            delegate_pk: self.delegate_pk,
            logs: self.logs.clone(),
            inflight: self.inflight,
            max_inflight,
            total_sent: self.total_sent,
            total_ok: self.total_ok,
            total_err: self.total_err,
            last_failure: self.last_failure.clone(),
            paused: self.paused,
            shutdown: self.shutdown,
            conservation_checks,
            tx_per_sec,
            success_rate,
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
    pub asset_summaries: Vec<AssetSummary>,
    pub delegate_pk: PublicKey,
    pub logs: VecDeque<String>,
    pub inflight: usize,
    pub max_inflight: usize,
    pub total_sent: u64,
    pub total_ok: u64,
    pub total_err: u64,
    pub last_failure: Option<String>,
    pub paused: bool,
    pub shutdown: bool,
    pub conservation_checks: u64,
    pub tx_per_sec: f64,
    pub success_rate: f64,
}

pub struct SimConfig {
    pub amount_range: (u64, u64),
    pub tick: Duration,
    pub max_inflight: usize,
}

/// Tracks expected per-asset supply and asserts conservation after every state change.
///
/// Once `seal()` is called (after bootstrap), the expected supply is frozen.
/// Every call to `assert_conservation()` sums all wallet balances plus inflight
/// amounts and panics with a full diagnosis if the totals don't match.
pub struct ConservationOracle {
    expected_supply: HashMap<Asset, u128>,
    sealed: bool,
    checks_passed: u64,
}

impl ConservationOracle {
    pub fn new() -> Self {
        Self {
            expected_supply: HashMap::new(),
            sealed: false,
            checks_passed: 0,
        }
    }

    /// Snapshot the current wallet totals as the expected supply. After this,
    /// any deviation is a bug.
    pub fn seal(&mut self, state: &AppState) {
        self.expected_supply.clear();
        for wallet in &state.wallets {
            for (asset, notes) in &wallet.notes {
                let total: u128 = notes.iter().map(|n| n.amount as u128).sum();
                *self.expected_supply.entry(*asset).or_default() += total;
            }
        }
        self.sealed = true;
    }

    /// Assert that the total supply across all wallets plus inflight amounts
    /// equals the expected supply. Panics with full diagnosis on violation.
    pub fn assert_conservation(
        &mut self,
        state: &AppState,
        inflight_amounts: &HashMap<Asset, u128>,
        context: &str,
    ) {
        if !self.sealed {
            return;
        }

        let mut actual: HashMap<Asset, u128> = HashMap::new();
        for wallet in &state.wallets {
            for (asset, notes) in &wallet.notes {
                let total: u128 = notes.iter().map(|n| n.amount as u128).sum();
                *actual.entry(*asset).or_default() += total;
            }
        }

        // Add inflight amounts
        for (asset, amount) in inflight_amounts {
            *actual.entry(*asset).or_default() += amount;
        }

        for (asset, &expected) in &self.expected_supply {
            let got = actual.get(asset).copied().unwrap_or(0);
            if got != expected {
                let wallet_detail: Vec<String> = state
                    .wallets
                    .iter()
                    .filter_map(|w| {
                        w.notes.get(asset).map(|notes| {
                            let total: u64 = notes.iter().map(|n| n.amount).sum();
                            format!(
                                "  wallet {}: {} notes, total {}",
                                w.id,
                                notes.len(),
                                total,
                            )
                        })
                    })
                    .collect();

                let inflight = inflight_amounts.get(asset).copied().unwrap_or(0);

                panic!(
                    "\n\n=== CONSERVATION VIOLATION ===\n\
                     context: {context}\n\
                     asset: {asset:?}\n\
                     expected supply: {expected}\n\
                     actual (wallets + inflight): {got}\n\
                     delta: {}\n\
                     inflight for asset: {inflight}\n\
                     checks passed before failure: {}\n\
                     wallet breakdown:\n{}\n\
                     ==============================\n",
                    got as i128 - expected as i128,
                    self.checks_passed,
                    wallet_detail.join("\n"),
                );
            }
        }

        // Check for unexpected new assets
        for (asset, &amount) in &actual {
            if amount > 0 && !self.expected_supply.contains_key(asset) {
                panic!(
                    "\n\n=== CONSERVATION VIOLATION ===\n\
                     context: {context}\n\
                     unexpected asset appeared: {asset:?}\n\
                     amount: {amount}\n\
                     checks passed before failure: {}\n\
                     ==============================\n",
                    self.checks_passed,
                );
            }
        }

        self.checks_passed += 1;
    }

    pub fn checks_passed(&self) -> u64 {
        self.checks_passed
    }
}

pub struct ThroughputTracker {
    window: Duration,
    ok_times: VecDeque<Instant>,
    err_times: VecDeque<Instant>,
}

impl ThroughputTracker {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            ok_times: VecDeque::new(),
            err_times: VecDeque::new(),
        }
    }

    pub fn record_ok(&mut self) {
        self.ok_times.push_back(Instant::now());
        self.prune();
    }

    pub fn record_err(&mut self) {
        self.err_times.push_back(Instant::now());
        self.prune();
    }

    fn prune(&mut self) {
        let cutoff = Instant::now() - self.window;
        while self.ok_times.front().is_some_and(|t| *t < cutoff) {
            self.ok_times.pop_front();
        }
        while self.err_times.front().is_some_and(|t| *t < cutoff) {
            self.err_times.pop_front();
        }
    }

    pub fn tx_per_sec(&mut self) -> f64 {
        self.prune();
        let total = self.ok_times.len() + self.err_times.len();
        total as f64 / self.window.as_secs_f64()
    }

    pub fn success_rate(&mut self) -> f64 {
        self.prune();
        let total = self.ok_times.len() + self.err_times.len();
        if total == 0 {
            return 100.0;
        }
        self.ok_times.len() as f64 / total as f64 * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct AssetSummary {
    pub name: &'static str,
    pub policy_id_hex: &'static str,
    pub total_supply: u64,
    pub total_notes: usize,
    pub wallets_holding: usize,
}

pub struct SimChannels {
    pub cmd_rx: tokio::sync::mpsc::UnboundedReceiver<SimCommand>,
    pub event_rx: tokio::sync::mpsc::UnboundedReceiver<SimEvent>,
    pub event_tx: tokio::sync::mpsc::UnboundedSender<SimEvent>,
    pub snapshot_tx: tokio::sync::watch::Sender<AppSnapshot>,
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
    pub input_amount: u64,
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
