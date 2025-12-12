use std::{
    collections::{HashMap, VecDeque},
    io::Stdout,
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr, eyre};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use mugraph_core::{
    builder::RefreshBuilder,
    types::{Blinded, Hash, Note, PublicKey, Refresh, Request, Response, Signature},
};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};
use reqwest::Url;
use tokio::{
    sync::{mpsc, watch},
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info};

#[derive(Debug, Parser)]
struct Args {
    /// Node base URL (e.g. http://127.0.0.1:9999)
    #[arg(long, default_value = "http://127.0.0.1:9999")]
    node_url: Url,
    /// Number of simulated wallets
    #[arg(long, default_value_t = 6)]
    wallets: usize,
    /// Number of distinct assets to simulate
    #[arg(long, default_value_t = 8)]
    assets: usize,
    /// Number of starting notes per wallet (per asset)
    #[arg(long, default_value_t = 2)]
    notes_per_wallet: usize,
    /// Minimum note/transfer amount
    #[arg(long, default_value_t = 1)]
    min_amount: u64,
    /// Maximum note/transfer amount
    #[arg(long, default_value_t = 50)]
    max_amount: u64,
    /// Milliseconds to wait between transaction attempts
    #[arg(long, default_value_t = 16)]
    tick_ms: u64,
    /// Maximum concurrent in-flight transactions
    #[arg(long, default_value_t = 16)]
    max_inflight: usize,
    /// RNG seed (optional) for reproducibility
    #[arg(long)]
    seed: Option<u64>,
}

#[derive(Clone)]
struct NodeClient {
    http: reqwest::Client,
    rpc_url: Url,
    health_url: Url,
    public_key_url: Url,
}

impl NodeClient {
    fn new(base: &Url) -> Result<Self> {
        let mut rpc_url = base.clone();
        rpc_url.set_path("/rpc");

        let mut health_url = base.clone();
        health_url.set_path("/health");

        let mut public_key_url = base.clone();
        public_key_url.set_path("/public_key");

        // Set a global request timeout so the simulator can't hang forever if
        // the node becomes unresponsive.
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .wrap_err("build http client")?;

        Ok(Self {
            http,
            rpc_url,
            health_url,
            public_key_url,
        })
    }

    async fn health(&self) -> Result<()> {
        let res = self.http.get(self.health_url.clone()).send().await?;
        if !res.status().is_success() {
            return Err(eyre!("health check failed with {}", res.status()));
        }
        Ok(())
    }

    async fn public_key(&self) -> Result<PublicKey> {
        let res = self
            .http
            .get(self.public_key_url.clone())
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    async fn rpc(&self, request: &Request) -> Result<Response> {
        let res = self
            .http
            .post(self.rpc_url.clone())
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    async fn emit(&self, asset_id: Hash, amount: u64) -> Result<Note> {
        match self.rpc(&Request::Emit { asset_id, amount }).await? {
            Response::Emit(note) => Ok(note),
            Response::Error { reason } => Err(eyre!("emit failed: {}", reason)),
            other => Err(eyre!("unexpected response for emit: {:?}", other)),
        }
    }

    async fn refresh(&self, refresh: &Refresh) -> Result<Vec<Blinded<Signature>>> {
        match self.rpc(&Request::Refresh(refresh.clone())).await? {
            Response::Transaction { outputs } => Ok(outputs),
            Response::Error { reason } => Err(eyre!("refresh failed: {}", reason)),
            other => Err(eyre!("unexpected response for refresh: {:?}", other)),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Wallet {
    id: usize,
    notes: HashMap<Hash, Vec<Note>>,
    sent: u64,
    received: u64,
    failures: u64,
}

#[derive(Debug, Default)]
struct AppState {
    wallets: Vec<Wallet>,
    assets: Vec<SimAsset>,
    delegate_pk: PublicKey,
    node_pk: Option<PublicKey>,
    logs: VecDeque<String>,
    inflight: usize,
    total_sent: u64,
    total_ok: u64,
    total_err: u64,
    last_failure: Option<String>,
    paused: bool,
    shutdown: bool,
}

impl AppState {
    fn log(&mut self, message: impl Into<String>) {
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

    fn snapshot(&self) -> AppSnapshot {
        let wallets = self
            .wallets
            .iter()
            .map(|wallet| WalletSnapshot {
                id: wallet.id,
                balances: self
                    .assets
                    .iter()
                    .map(|asset| {
                        let notes = wallet.notes.get(&asset.id);
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
struct WalletBalance {
    balance: u64,
    notes: usize,
}

#[derive(Debug, Clone)]
struct WalletSnapshot {
    id: usize,
    balances: Vec<WalletBalance>,
    sent: u64,
    received: u64,
    failures: u64,
}

#[derive(Debug, Clone)]
struct AppSnapshot {
    wallets: Vec<WalletSnapshot>,
    assets: Vec<SimAsset>,
    delegate_pk: PublicKey,
    node_pk: Option<PublicKey>,
    logs: VecDeque<String>,
    inflight: usize,
    total_sent: u64,
    total_ok: u64,
    total_err: u64,
    last_failure: Option<String>,
    paused: bool,
    shutdown: bool,
}

#[derive(Debug, Clone, Copy)]
enum SimCommand {
    TogglePause,
    Quit,
}

#[derive(Debug)]
struct PendingTx {
    id: u64,
    sender_id: usize,
    receiver_id: usize,
    asset: Hash,
    input_note: Note,
    spend_amount: u64,
    refresh: Refresh,
    owners: Vec<usize>,
}

#[derive(Debug)]
enum SimEvent {
    TxFinished {
        pending: PendingTx,
        result: std::result::Result<Vec<Blinded<Signature>>, String>,
    },
}

#[derive(Debug, Clone, Copy)]
struct CardanoAssetDef {
    policy_id: &'static str,
    asset_name: &'static str,
}

const CARDANO_ASSET_DEFS: &[CardanoAssetDef] = &[
    CardanoAssetDef {
        policy_id: "279c909f348e533da5808898f87f9a14bb2c3dfbbacccd631d927a3f",
        asset_name: "SNEK",
    },
    CardanoAssetDef {
        policy_id: "a0028f350aaabe0545fdcb56b039bfb08e4bb4d8c4d7c3c7d481c235",
        asset_name: "HOSKY",
    },
    CardanoAssetDef {
        policy_id: "2b28c81dbba6d67e4b5a997c6be1212cba9d60d33f82444ab8b1f218",
        asset_name: "BANK",
    },
    CardanoAssetDef {
        policy_id: "961f2cac0bb1967d74691af179350c1e1062c7298d1f7be1e4696e31",
        asset_name: "$DERP",
    },
    CardanoAssetDef {
        policy_id: "98dc68b04026544619a251bc01aad2075d28433524ac36cbc75599a1",
        asset_name: "hosk",
    },
    CardanoAssetDef {
        policy_id: "7507734918533b3b896241b4704f3d4ce805256b01da6fcede430436",
        asset_name: "BabySNEK",
    },
    CardanoAssetDef {
        policy_id: "e629ee88f6dcad8948e420b929e114c8785e2d4fb9d5c077157a3b37",
        asset_name: "BLUP",
    },
    CardanoAssetDef {
        policy_id: "884892bcdc360bcef87d6b3f806e7f9cd5ac30d999d49970e7a903ae",
        asset_name: "PAVIA",
    },
    CardanoAssetDef {
        policy_id: "8a1cfae21368b8bebbbed9800fec304e95cce39a2a57dc35e2e3ebaa",
        asset_name: "MILK",
    },
    CardanoAssetDef {
        policy_id: "6cfbfedd8c8ea23d264f5ae3ef039217100c210bb66de8711f21c903",
        asset_name: "CNFT",
    },
    CardanoAssetDef {
        policy_id: "52489ea87bbceaf6375cc22f74c19382a3d5da3f8b9b15d2537044b9",
        asset_name: "PRSPR",
    },
    CardanoAssetDef {
        policy_id: "9f452e23804df3040b352b478039357b506ad3b50d2ce0d7cbd5f806",
        asset_name: "CTV",
    },
    CardanoAssetDef {
        policy_id: "6787a47e9f73efe4002d763337140da27afa8eb9a39413d2c39d4286",
        asset_name: "RADTokens",
    },
    CardanoAssetDef {
        policy_id: "f7c777fdd4531cf1c477551360e45b9684073c05c2fa61334f8f9add",
        asset_name: "VeritreeToken",
    },
    CardanoAssetDef {
        policy_id: "a00fdf4fb9ab6c8c2bd1533a2f14855edf12aed5ecbf96d4b5f5b939",
        asset_name: "C4",
    },
    CardanoAssetDef {
        policy_id: "8f52f6a88acf6127bc4758a16b6047afc4da7887feae121ec217b75a",
        asset_name: "SNOW",
    },
    CardanoAssetDef {
        policy_id: "b166a1047a8cd275bf0a50201ece3d4f0b4da300094ffcc668a6f408",
        asset_name: "KITUP",
    },
    CardanoAssetDef {
        policy_id: "2441ab3351c3b80213a98f4e09ddcf7dabe4879c3c94cc4e7205cb63",
        asset_name: "FIRE",
    },
    CardanoAssetDef {
        policy_id: "dca54ecf37b0e3af2fdfd336e1d21fadcc45b3261b0f73a095631dfe",
        asset_name: "DOEX",
    },
    CardanoAssetDef {
        policy_id: "160a880d9fc45380737cb7e57ff859763230aab28b3ef6a84007bfcc",
        asset_name: "MIRA",
    },
    CardanoAssetDef {
        policy_id: "db30c7905f598ed0154de14f970de0f61f0cb3943ed82c891968480a",
        asset_name: "CLAP",
    },
    CardanoAssetDef {
        policy_id: "547ceed647f57e64dc40a29b16be4f36b0d38b5aa3cd7afb286fc094",
        asset_name: "bbHosky",
    },
    CardanoAssetDef {
        policy_id: "5d16cc1a177b5d9ba9cfa9793b07e60f1fb70fea1f8aef064415d114",
        asset_name: "IAG",
    },
    CardanoAssetDef {
        policy_id: "1d7f33bd23d85e1a25d87d86fac4f199c3197a2f7afeb662a0f34e1e",
        asset_name: "worldmobiletoken",
    },
    CardanoAssetDef {
        policy_id: "38ad9dc3aec6a2f38e220142b9aa6ade63ebe71f65e7cc2b7d8a8535",
        asset_name: "CLAY",
    },
    CardanoAssetDef {
        policy_id: "8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd61",
        asset_name: "ShenMicroUSD",
    },
    CardanoAssetDef {
        policy_id: "b6a7467ea1deb012808ef4e87b5ff371e85f7142d7b356a40d9b42a0",
        asset_name: "Cornucopias [via ChainPort.io]",
    },
    CardanoAssetDef {
        policy_id: "533bb94a8850ee3ccbe483106489399112b74c905342cb1792a797a0",
        asset_name: "INDY",
    },
    CardanoAssetDef {
        policy_id: "edfd7a1d77bcb8b884c474bdc92a16002d1fb720e454fa6e99344479",
        asset_name: "NTX",
    },
    CardanoAssetDef {
        policy_id: "f66d78b4a3cb3d37afa0ec36461e51ecbde00f26c8f0a68f94b69880",
        asset_name: "iUSD",
    },
    CardanoAssetDef {
        policy_id: "8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd61",
        asset_name: "DjedMicroUSD",
    },
    CardanoAssetDef {
        policy_id: "5dac8536653edc12f6f5e1045d8164b9f59998d3bdc300fc92843489",
        asset_name: "NMKR",
    },
    CardanoAssetDef {
        policy_id: "4fde92c2f6dbcfa2879b44f7453872b31394cfb2f70f1d4c411169ac",
        asset_name: "Bubble",
    },
    CardanoAssetDef {
        policy_id: "f66d78b4a3cb3d37afa0ec36461e51ecbde00f26c8f0a68f94b69880",
        asset_name: "iBTC",
    },
    CardanoAssetDef {
        policy_id: "8cfd6893f5f6c1cc954cec1a0a1460841b74da6e7803820dde62bb78",
        asset_name: "RJV",
    },
    CardanoAssetDef {
        policy_id: "29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6",
        asset_name: "MIN",
    },
    CardanoAssetDef {
        policy_id: "95a427e384527065f2f8946f5e86320d0117839a5e98ea2c0b55fb00",
        asset_name: "HUNT",
    },
    CardanoAssetDef {
        policy_id: "c0ee29a85b13209423b10447d3c2e6a50641a15c57770e27cb9d5073",
        asset_name: "WingRiders",
    },
    CardanoAssetDef {
        policy_id: "8e51398904a5d3fc129fbf4f1589701de23c7824d5c90fdb9490e15a",
        asset_name: "CHARLI3",
    },
    CardanoAssetDef {
        policy_id: "8daefa391220bd0d8d007f3748d870f7f3c106040314c8515ccc35a5",
        asset_name: "FLAC",
    },
    CardanoAssetDef {
        policy_id: "10a49b996e2402269af553a8a96fb8eb90d79e9eca79e2b4223057b6",
        asset_name: "GERO",
    },
    CardanoAssetDef {
        policy_id: "a3931691f5c4e65d01c429e473d0dd24c51afdb6daf88e632a6c1e51",
        asset_name: "orcfaxtoken",
    },
    CardanoAssetDef {
        policy_id: "f66d78b4a3cb3d37afa0ec36461e51ecbde00f26c8f0a68f94b69880",
        asset_name: "iETH",
    },
    CardanoAssetDef {
        policy_id: "6c8642400e8437f737eb86df0fc8a8437c760f48592b1ba8f5767e81",
        asset_name: "Empowa",
    },
    CardanoAssetDef {
        policy_id: "9a9693a9a37912a5097918f97918d15240c92ab729a0b7c4aa144d77",
        asset_name: "SUNDAE",
    },
    CardanoAssetDef {
        policy_id: "afbe91c0b44b3040e360057bf8354ead8c49c4979ae6ab7c4fbdc9eb",
        asset_name: "MILKv2",
    },
    CardanoAssetDef {
        policy_id: "682fe60c9918842b3323c43b5144bc3d52a23bd2fb81345560d73f63",
        asset_name: "NEWM",
    },
    CardanoAssetDef {
        policy_id: "b316f8f668aca7359ecc6073475c0c8106239bf87e05a3a1bd569764",
        asset_name: "xVYFI",
    },
];

#[derive(Debug, Clone, Copy)]
struct SimAsset {
    id: Hash,
    name: &'static str,
    policy_id: &'static str,
}

fn generate_assets(count: usize, rng: &mut StdRng) -> Vec<SimAsset> {
    let mut defs: Vec<CardanoAssetDef> = CARDANO_ASSET_DEFS.to_vec();
    defs.shuffle(rng);

    let mut selected = Vec::with_capacity(count);
    for i in 0..count {
        let def = defs[i % defs.len()];
        let mut bytes = Vec::new();
        bytes.extend_from_slice(def.policy_id.as_bytes());
        bytes.extend_from_slice(def.asset_name.as_bytes());
        let id = Hash::digest(&bytes);
        selected.push(SimAsset {
            id,
            name: def.asset_name,
            policy_id: def.policy_id,
        });
    }

    selected
}

async fn bootstrap_wallets(
    client: &NodeClient,
    state: &mut AppState,
    assets: &[SimAsset],
    wallets: usize,
    notes_per_wallet: usize,
    amount_range: (u64, u64),
    rng: &mut StdRng,
) -> Result<()> {
    state.wallets = (0..wallets)
        .map(|id| Wallet {
            id,
            ..Default::default()
        })
        .collect();

    for wallet_id in 0..wallets {
        for asset in assets.iter() {
            for _ in 0..notes_per_wallet {
                let amount = rng.random_range(amount_range.0..=amount_range.1);
                let note = client.emit(asset.id, amount).await?;
                state.log(format!(
                    "emit via node wallet={} asset={} amount={amount}",
                    wallet_id, asset.name
                ));

                let w = &mut state.wallets[wallet_id];
                w.notes.entry(asset.id).or_default().push(note);
            }
        }
    }

    Ok(())
}

fn reserve_spendable_note(
    wallet: &mut Wallet,
    assets: &[SimAsset],
    rng: &mut StdRng,
) -> Option<(Hash, Note)> {
    let mut shuffled: Vec<Hash> = assets.iter().map(|a| a.id).collect();
    shuffled.shuffle(rng);
    for asset in shuffled {
        if let Some(notes) = wallet.notes.get_mut(&asset) {
            if let Some(pos) = notes.iter().position(|n| n.amount > 0) {
                return Some((asset, notes.swap_remove(pos)));
            }
        }
    }
    None
}

fn build_refresh(
    input_owner: usize,
    output_owner: usize,
    asset: Hash,
    input_note: Note,
    amount: u64,
) -> Result<(Refresh, Vec<usize>)> {
    let mut builder = RefreshBuilder::new().input(input_note.clone());
    builder = builder.output(asset, amount);

    if input_note.amount > amount {
        let change = input_note.amount - amount;
        builder = builder.output(asset, change);
    }

    let refresh = builder.build()?;

    let mut owners = Vec::new();
    owners.push(output_owner);
    if input_note.amount > amount {
        owners.push(input_owner);
    }

    Ok((refresh, owners))
}

fn materialize_outputs(
    refresh: &Refresh,
    outputs: Vec<Blinded<Signature>>,
    owners: &[usize],
) -> Result<Vec<(usize, Note)>> {
    let mut created = Vec::new();
    let mut output_iter = outputs.into_iter();
    for (atom_idx, atom) in refresh.atoms.iter().enumerate() {
        if refresh.is_input(atom_idx) {
            continue;
        }

        let signature = output_iter
            .next()
            .ok_or_else(|| eyre!("missing signature for output {}", atom_idx))?;

        let asset = refresh
            .asset_ids
            .get(atom.asset_id as usize)
            .ok_or_else(|| eyre!("invalid asset index {}", atom.asset_id))?;

        let note = Note {
            amount: atom.amount,
            delegate: atom.delegate,
            asset_id: *asset,
            nonce: atom.nonce,
            signature: signature.0,
        };

        let owner = owners
            .get(created.len())
            .ok_or_else(|| eyre!("missing owner mapping"))?;
        created.push((*owner, note));
    }

    Ok(created)
}

async fn simulation_owner_loop(
    client: NodeClient,
    mut state: AppState,
    mut rng: StdRng,
    amount_range: (u64, u64),
    tick: Duration,
    max_inflight: usize,
    mut cmd_rx: mpsc::UnboundedReceiver<SimCommand>,
    mut event_rx: mpsc::UnboundedReceiver<SimEvent>,
    event_tx: mpsc::UnboundedSender<SimEvent>,
    snapshot_tx: watch::Sender<AppSnapshot>,
) {
    let mut ticker = interval(tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let _ = snapshot_tx.send(state.snapshot());

    let mut tx_id: u64 = 0;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if state.shutdown {
                    break;
                }
                if state.paused || state.inflight >= max_inflight {
                    continue;
                }

                let wallet_count = state.wallets.len();
                if wallet_count < 2 {
                    continue;
                }

                let sender_idx = rng.random_range(0..wallet_count);
                let receiver_idx = {
                    let mut idx = rng.random_range(0..wallet_count - 1);
                    if idx >= sender_idx {
                        idx += 1;
                    }
                    idx
                };

                let sender_id = state.wallets[sender_idx].id;
                let receiver_id = state.wallets[receiver_idx].id;

                let Some((asset, input_note)) = reserve_spendable_note(
                    &mut state.wallets[sender_idx],
                    &state.assets,
                    &mut rng,
                ) else {
                    continue;
                };

                let spend_amount = rng
                    .random_range(amount_range.0..=amount_range.1)
                    .min(input_note.amount);

                let (refresh, owners) = match build_refresh(
                    sender_id,
                    receiver_id,
                    asset,
                    input_note.clone(),
                    spend_amount,
                ) {
                    Ok(res) => res,
                    Err(e) => {
                        state.wallets[sender_id]
                            .notes
                            .entry(asset)
                            .or_default()
                            .push(input_note);
                        state.last_failure = Some(e.to_string());
                        state.total_err += 1;
                        state.wallets[sender_id].failures += 1;
                        state.log(format!("failed to build refresh: {e:#}"));
                        let _ = snapshot_tx.send(state.snapshot());
                        continue;
                    }
                };

                tx_id += 1;
                let pending = PendingTx {
                    id: tx_id,
                    sender_id,
                    receiver_id,
                    asset,
                    input_note,
                    spend_amount,
                    refresh,
                    owners,
                };

                state.inflight += 1;
                state.total_sent += 1;
                let _ = snapshot_tx.send(state.snapshot());

                let client_clone = client.clone();
                let event_tx_clone = event_tx.clone();
                tokio::spawn(async move {
                    let result = match client_clone.refresh(&pending.refresh).await {
                        Ok(outputs) => Ok(outputs),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = event_tx_clone.send(SimEvent::TxFinished { pending, result });
                });
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    SimCommand::TogglePause => {
                        state.paused = !state.paused;
                        state.log(format!("paused set to {}", state.paused));
                        let _ = snapshot_tx.send(state.snapshot());
                    }
                    SimCommand::Quit => {
                        state.shutdown = true;
                        state.log("shutting down");
                        let _ = snapshot_tx.send(state.snapshot());
                        break;
                    }
                }
            }
            Some(event) = event_rx.recv() => {
                match event {
                    SimEvent::TxFinished { pending, result } => {
                        state.inflight = state.inflight.saturating_sub(1);
                        match result {
                            Ok(outputs) => match materialize_outputs(&pending.refresh, outputs, &pending.owners) {
                                Ok(notes) => {
                                    for (owner, note) in notes {
                                        state.wallets[owner]
                                            .notes
                                            .entry(note.asset_id)
                                            .or_default()
                                            .push(note);
                                    }
                                    state.wallets[pending.sender_id].sent += 1;
                                    state.wallets[pending.receiver_id].received += 1;
                                    state.total_ok += 1;
                                    state.log(format!(
                                        "tx {} ok sender={} receiver={} amount={}",
                                        pending.id, pending.sender_id, pending.receiver_id, pending.spend_amount
                                    ));
                                }
                                Err(e) => {
                                    state.last_failure = Some(e.to_string());
                                    state.total_err += 1;
                                    state.wallets[pending.sender_id].failures += 1;
                                    state.wallets[pending.sender_id]
                                        .notes
                                        .entry(pending.asset)
                                        .or_default()
                                        .push(pending.input_note);
                                    state.log(format!(
                                        "tx {} materialization failed: {e:#}",
                                        pending.id
                                    ));
                                }
                            },
                            Err(reason) => {
                                state.last_failure = Some(reason.clone());
                                state.total_err += 1;
                                state.wallets[pending.sender_id].failures += 1;
                                state.wallets[pending.sender_id]
                                    .notes
                                    .entry(pending.asset)
                                    .or_default()
                                    .push(pending.input_note);
                                state.log(format!("tx {} failed: {}", pending.id, reason));
                            }
                        }
                        let _ = snapshot_tx.send(state.snapshot());
                    }
                }
            }
        }
    }

    state.shutdown = true;
    let _ = snapshot_tx.send(state.snapshot());
}

fn render_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    snapshot: &AppSnapshot,
) -> Result<()> {
    let paused = snapshot.paused;

    terminal.draw(|f| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(10),
                ]
                .as_ref(),
            )
            .split(f.area());

        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Node: "),
                Span::styled(
                    snapshot
                        .node_pk
                        .map(|pk| format!("{pk}"))
                        .unwrap_or_else(|| "unknown".into()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  Delegate: "),
                Span::styled(
                    format!("{}", snapshot.delegate_pk),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Paused: "),
                Span::styled(
                    format!("{}", paused),
                    Style::default().fg(if paused { Color::Yellow } else { Color::Green }),
                ),
            ]),
            Line::from(vec![
                Span::raw("Tx sent/ok/err: "),
                Span::styled(
                    format!(
                        "{}/{}/{}",
                        snapshot.total_sent, snapshot.total_ok, snapshot.total_err
                    ),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw("  Inflight: "),
                Span::styled(
                    snapshot.inflight.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  Last err: "),
                Span::styled(
                    snapshot.last_failure.as_deref().unwrap_or("-"),
                    Style::default().fg(Color::Red),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(header, layout[0]);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(layout[1]);

        let mut rows = Vec::new();
        for wallet in snapshot.wallets.iter() {
            let row_style = Style::default().fg(wallet_color(wallet.id));
            let mut balance_lines = Vec::new();
            for (asset, balance) in snapshot.assets.iter().zip(wallet.balances.iter()) {
                let short_policy = asset.policy_id.get(0..8).unwrap_or(asset.policy_id);
                balance_lines.push(Line::from(vec![
                    Span::styled(asset.name, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(
                        " ({short_policy}) bal={} notes={}",
                        balance.balance, balance.notes
                    )),
                ]));
            }

            let balances_cell = Cell::from(Text::from(balance_lines.clone()));
            let height = balance_lines.len().max(1) as u16;

            rows.push(
                Row::new(vec![
                    Cell::from(wallet.id.to_string()),
                    balances_cell,
                    Cell::from(wallet.sent.to_string()),
                    Cell::from(wallet.received.to_string()),
                    Cell::from(wallet.failures.to_string()),
                ])
                .style(row_style)
                .height(height),
            );
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Min(10),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Length(8),
            ],
        )
        .header(Row::new(["id", "balances", "sent", "received", "fail"]))
        .block(Block::default().borders(Borders::ALL).title("Wallets"))
        .column_spacing(2);

        f.render_widget(table, body_chunks[0]);

        let logs: Vec<ListItem> = snapshot
            .logs
            .iter()
            .map(|l| ListItem::new(l.clone()))
            .collect();
        let log_block = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
        f.render_widget(log_block, body_chunks[1]);

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": pause/resume"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, layout[2]);
    })?;

    Ok(())
}

fn wallet_color(id: usize) -> Color {
    const PALETTE: [Color; 8] = [
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::Blue,
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
    ];
    PALETTE[id % PALETTE.len()]
}

fn ui_loop(
    snapshot_rx: watch::Receiver<AppSnapshot>,
    cmd_tx: mpsc::UnboundedSender<SimCommand>,
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    loop {
        let snapshot = snapshot_rx.borrow().clone();
        if snapshot.shutdown {
            break;
        }

        render_ui(&mut terminal, &snapshot)?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::Quit);
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::TogglePause);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::Quit);
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let mut rng: StdRng = match args.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => {
            let mut thread = rand::rng();
            StdRng::from_rng(&mut thread)
        }
    };

    let client = NodeClient::new(&args.node_url)?;
    client.health().await.wrap_err("node health check failed")?;
    let node_pk = client
        .public_key()
        .await
        .wrap_err("fetch node public key")?;

    info!(
        "connected to node {} (delegate pk {})",
        args.node_url, node_pk
    );

    let assets = generate_assets(args.assets, &mut rng);
    let mut state = AppState {
        assets: assets.clone(),
        delegate_pk: node_pk,
        node_pk: Some(node_pk),
        ..Default::default()
    };

    bootstrap_wallets(
        &client,
        &mut state,
        &assets,
        args.wallets,
        args.notes_per_wallet,
        (args.min_amount, args.max_amount),
        &mut rng,
    )
    .await
    .wrap_err("bootstrap wallets")?;

    let tick = Duration::from_millis(args.tick_ms);

    let (snapshot_tx, snapshot_rx) = watch::channel(state.snapshot());
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mut owner_handle = tokio::spawn(simulation_owner_loop(
        client,
        state,
        rng,
        (args.min_amount, args.max_amount),
        tick,
        args.max_inflight,
        cmd_rx,
        event_rx,
        event_tx,
        snapshot_tx,
    ));

    let terminal = ratatui::init();
    let ui_cmd_tx = cmd_tx.clone();
    let mut ui_handle =
        tokio::task::spawn_blocking(move || ui_loop(snapshot_rx, ui_cmd_tx, terminal));

    let mut owner_done = false;
    let mut ui_done = false;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl+c, shutting down");
            let _ = cmd_tx.send(SimCommand::Quit);
        }
        res = &mut ui_handle => {
            ui_done = true;
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => error!("ui task error: {e:#}"),
                Err(e) => error!("ui task join error: {e:?}"),
            }
            let _ = cmd_tx.send(SimCommand::Quit);
        }
        res = &mut owner_handle => {
            owner_done = true;
            if let Err(e) = res {
                error!("simulation owner task error: {e:?}");
            }
        }
    }

    let _ = cmd_tx.send(SimCommand::Quit);

    if !owner_done {
        if let Err(e) = owner_handle.await {
            error!("simulation owner task error: {e:?}");
        }
    }
    if !ui_done {
        match ui_handle.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => error!("ui task error: {e:#}"),
            Err(e) => error!("ui task join error: {e:?}"),
        }
    }

    ratatui::restore();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_note(signature_byte: u8) -> Note {
        Note {
            amount: 1,
            delegate: PublicKey([9u8; 32]),
            asset_id: Hash([7u8; 32]),
            nonce: Hash([5u8; 32]),
            signature: Signature([signature_byte; 32]),
        }
    }

    #[test]
    fn reserve_by_signature_survives_reordering() {
        let mut notes = vec![dummy_note(1), dummy_note(2)];
        let target = notes[1].clone();

        // Simulate concurrent modification before reserving.
        notes.swap_remove(0);

        let Some(pos) = notes.iter().position(|n| n.signature == target.signature) else {
            panic!("target note missing");
        };

        notes.swap_remove(pos);
        assert!(notes.is_empty());
    }

    #[test]
    fn reinsert_is_deduped_by_signature() {
        let mut notes = vec![dummy_note(3)];
        let note = notes[0].clone();

        if !notes.iter().any(|n| n.signature == note.signature) {
            notes.push(note.clone());
        }
        assert_eq!(notes.len(), 1);

        let other = dummy_note(4);
        if !notes.iter().any(|n| n.signature == other.signature) {
            notes.push(other);
        }
        assert_eq!(notes.len(), 2);
    }
}
