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
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
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
    #[arg(long, default_value_t = 2)]
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
    #[arg(long, default_value_t = 400)]
    tick_ms: u64,
    /// Maximum concurrent in-flight transactions
    #[arg(long, default_value_t = 4)]
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
    assets: Vec<Hash>,
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

fn generate_assets(rng: &mut impl Rng, count: usize) -> Vec<Hash> {
    (0..count)
        .map(|i| {
            let mut buf = [0u8; 32];
            rng.fill(buf.as_mut_slice());
            let mut tag = format!("asset-{i}-").into_bytes();
            tag.resize(32, 0);
            let digest = Hash::digest(&buf);
            let mut combined = [0u8; 32];
            for (i, (a, b)) in tag.iter().zip(digest.as_ref()).enumerate() {
                combined[i] = a ^ b;
            }
            Hash::from(combined)
        })
        .collect()
}

async fn bootstrap_wallets(
    client: &NodeClient,
    state: &SharedState,
    assets: &[Hash],
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
                let note = client.emit(*asset, amount).await?;
                state.log(format!(
                    "emit via node wallet={} asset={asset:?} amount={amount}",
                    wallet_id
                ));

                let mut guard = state.inner.write().unwrap();
                let w = &mut guard.wallets[wallet_id];
                w.notes.entry(*asset).or_default().push(note);
            }
        }
    }

    Ok(())
}

fn choose_spendable_note(
    wallet: &Wallet,
    assets: &[Hash],
    rng: &mut StdRng,
) -> Option<(Hash, usize, Note)> {
    let mut shuffled = assets.to_vec();
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
            let mut balance_strings = Vec::new();
            for asset in snapshot.assets.iter() {
                let bal = wallet.total_balance(asset);
                balance_strings.push(format!("{bal}"));
            }
            rows.push(Row::new(vec![
                wallet.id.to_string(),
                balance_strings.join(","),
                wallet.sent.to_string(),
                wallet.received.to_string(),
                wallet.failures.to_string(),
            ]));
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

    let assets = generate_assets(&mut rng, args.assets);
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
