use std::{
    collections::{HashMap, VecDeque},
    io::Stdout,
    sync::{
        Arc,
        RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
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
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, info, warn};

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

impl Wallet {
    fn total_balance(&self, asset: &Hash) -> u64 {
        self.notes
            .get(asset)
            .map(|v| v.iter().map(|n| n.amount).sum())
            .unwrap_or(0)
    }
}

#[derive(Debug)]
struct InflightTx {
    refresh: Refresh,
    output_owners: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
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
}

struct SharedState {
    inner: RwLock<AppState>,
    paused: AtomicBool,
}

impl SharedState {
    fn new(app: AppState) -> Self {
        Self {
            inner: RwLock::new(app),
            paused: AtomicBool::new(false),
        }
    }

    fn log(&self, message: impl Into<String>) {
        let mut guard = self.inner.write().unwrap();
        let entry = message.into();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        guard.logs.push_front(format!(
            "[{:>6}.{:03}] {}",
            now.as_secs(),
            now.subsec_millis(),
            entry
        ));
        if guard.logs.len() > 200 {
            guard.logs.pop_back();
        }
    }
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
    state: &SharedState,
    assets: &[SimAsset],
    wallets: usize,
    notes_per_wallet: usize,
    amount_range: (u64, u64),
    rng: &mut StdRng,
) -> Result<()> {
    {
        let mut guard = state.inner.write().unwrap();
        guard.wallets = (0..wallets)
            .map(|id| Wallet {
                id,
                ..Default::default()
            })
            .collect();
    }

    for wallet_id in 0..wallets {
        for asset in assets.iter() {
            for _ in 0..notes_per_wallet {
                let amount = rng.random_range(amount_range.0..=amount_range.1);
                let note = client.emit(asset.id, amount).await?;
                state.log(format!(
                    "emit via node wallet={} asset={} amount={amount}",
                    wallet_id, asset.name
                ));

                let mut guard = state.inner.write().unwrap();
                let w = &mut guard.wallets[wallet_id];
                w.notes.entry(asset.id).or_default().push(note);
            }
        }
    }

    Ok(())
}

fn choose_spendable_note(
    wallet: &Wallet,
    assets: &[SimAsset],
    rng: &mut StdRng,
) -> Option<(Hash, usize, Note)> {
    let mut shuffled: Vec<Hash> = assets.iter().map(|a| a.id).collect();
    shuffled.shuffle(rng);
    for asset in shuffled {
        if let Some(notes) = wallet.notes.get(&asset) {
            if let Some((idx, note)) = notes.iter().enumerate().find(|(_, n)| n.amount > 0) {
                return Some((asset, idx, note.clone()));
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

async fn traffic_loop(
    client: NodeClient,
    shared: Arc<SharedState>,
    mut rng: StdRng,
    amount_range: (u64, u64),
    tick: Duration,
    max_inflight: usize,
) {
    let inflight_counter = Arc::new(AtomicU64::new(0));
    let mut tx_id: u64 = 0;

    loop {
        if shared.paused.load(Ordering::SeqCst) {
            sleep(tick).await;
            continue;
        }

        let (snapshot, asset_list) = {
            let guard = shared.inner.read().unwrap();
            (guard.wallets.clone(), guard.assets.clone())
        };
        if snapshot.is_empty() {
            sleep(tick).await;
            continue;
        }

        if inflight_counter.load(Ordering::SeqCst) >= max_inflight as u64 {
            sleep(tick).await;
            continue;
        }

        let sender_idx = rng.random_range(0..snapshot.len());
        let sender = snapshot[sender_idx].clone();

        let Some((asset, note_idx, note)) = choose_spendable_note(&sender, &asset_list, &mut rng)
        else {
            sleep(tick).await;
            continue;
        };

        let mut receivers: Vec<_> = snapshot.iter().filter(|w| w.id != sender.id).collect();
        receivers.shuffle(&mut rng);
        let Some(receiver) = receivers.first().map(|w| w.id) else {
            sleep(tick).await;
            continue;
        };

        let spend_amount = rng
            .random_range(amount_range.0..=amount_range.1)
            .min(note.amount);

        let (refresh, owners) =
            match build_refresh(sender.id, receiver, asset, note.clone(), spend_amount) {
                Ok(res) => res,
                Err(e) => {
                    error!("failed to build refresh: {e:#}");
                    sleep(tick).await;
                    continue;
                }
            };

        // Reserve input note
        {
            let mut guard = shared.inner.write().unwrap();
            if let Some(notes) = guard.wallets[sender.id].notes.get_mut(&asset) {
                if note_idx < notes.len() {
                    notes.swap_remove(note_idx);
                }
            }
            guard.inflight += 1;
            inflight_counter.fetch_add(1, Ordering::SeqCst);
            guard.total_sent += 1;
        }

        let shared_clone = shared.clone();
        let inflight_counter_clone = inflight_counter.clone();
        let client_clone = client.clone();
        tx_id += 1;
        let id = tx_id;

        tokio::spawn(async move {
            let inflight = InflightTx {
                refresh: refresh.clone(),
                output_owners: owners.clone(),
            };
            match client_clone.refresh(&refresh).await {
                Ok(outputs) => {
                    let notes = match materialize_outputs(
                        &inflight.refresh,
                        outputs,
                        &inflight.output_owners,
                    ) {
                        Ok(n) => n,
                        Err(e) => {
                            error!("tx {id} materialization failed: {e:#}");
                            let mut guard = shared_clone.inner.write().unwrap();
                            guard.last_failure = Some(e.to_string());
                            guard.wallets[sender.id]
                                .notes
                                .entry(asset)
                                .or_default()
                                .push(note.clone());
                            guard.inflight = guard.inflight.saturating_sub(1);
                            guard.total_err += 1;
                            inflight_counter_clone.fetch_sub(1, Ordering::SeqCst);
                            return;
                        }
                    };

                    let log_msg = {
                        let mut guard = shared_clone.inner.write().unwrap();
                        for (owner, note) in notes {
                            guard.wallets[owner]
                                .notes
                                .entry(note.asset_id)
                                .or_default()
                                .push(note.clone());
                        }
                        guard.wallets[sender.id].sent += 1;
                        guard.wallets[receiver].received += 1;
                        guard.inflight = guard.inflight.saturating_sub(1);
                        guard.total_ok += 1;
                        format!(
                            "tx {id} ok sender={} receiver={} amount={spend_amount}",
                            sender.id, receiver
                        )
                    };
                    shared_clone.log(log_msg);
                }
                Err(e) => {
                    warn!("tx {id} failed: {e:#}");
                    let log_msg = format!("tx {id} failed: {e:#}");
                    {
                        let mut guard = shared_clone.inner.write().unwrap();
                        guard.last_failure = Some(e.to_string());
                        guard.wallets[sender.id]
                            .notes
                            .entry(asset)
                            .or_default()
                            .push(note.clone());
                        guard.wallets[sender.id].failures += 1;
                        guard.inflight = guard.inflight.saturating_sub(1);
                        guard.total_err += 1;
                    }
                    shared_clone.log(log_msg);
                }
            }

            inflight_counter_clone.fetch_sub(1, Ordering::SeqCst);
        });

        sleep(tick).await;
    }
}

fn render_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, shared: &SharedState) -> Result<()> {
    let snapshot = shared.inner.read().unwrap().clone();
    let paused = shared.paused.load(Ordering::SeqCst);

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
            for asset in snapshot.assets.iter() {
                let bal = wallet.total_balance(&asset.id);
                let count = wallet.notes.get(&asset.id).map(|v| v.len()).unwrap_or(0);
                let short_policy = asset.policy_id.get(0..8).unwrap_or(asset.policy_id);
                balance_lines.push(Line::from(vec![
                    Span::styled(asset.name, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" ({short_policy}) bal={bal} notes={count}")),
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

fn ui_loop(shared: Arc<SharedState>, mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    loop {
        render_ui(&mut terminal, &shared)?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                }) => {
                    let paused = shared.paused.fetch_xor(true, Ordering::SeqCst);
                    shared.log(format!("paused set to {}", !paused));
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break,
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
    let state = Arc::new(SharedState::new(AppState {
        assets: assets.clone(),
        delegate_pk: node_pk,
        node_pk: Some(node_pk),
        ..Default::default()
    }));

    bootstrap_wallets(
        &client,
        &state,
        &assets,
        args.wallets,
        args.notes_per_wallet,
        (args.min_amount, args.max_amount),
        &mut rng,
    )
    .await
    .wrap_err("bootstrap wallets")?;

    let traffic_state = state.clone();
    let traffic_client = client;
    let traffic_rng = rng;
    let tick = Duration::from_millis(args.tick_ms);
    let traffic_handle: JoinHandle<()> = tokio::spawn(async move {
        traffic_loop(
            traffic_client,
            traffic_state,
            traffic_rng,
            (args.min_amount, args.max_amount),
            tick,
            args.max_inflight,
        )
        .await;
    });

    let terminal = ratatui::init();
    let ui_state = state.clone();
    let ui_handle = tokio::task::spawn_blocking(move || ui_loop(ui_state, terminal));

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl+c, shutting down");
        }
        res = ui_handle => {
            if let Err(e) = res {
                error!("ui task error: {e:?}");
            }
        }
    }

    traffic_handle.abort();
    ratatui::restore();
    Ok(())
}
