mod assets;
mod client;
mod simulation;
mod types;
mod ui;

use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use rand::{SeedableRng, rngs::StdRng};
use tokio::sync::{mpsc, watch};
use tracing::{error, info};

use crate::{
    assets::generate_assets,
    client::NodeClient,
    simulation::{bootstrap_wallets, simulation_owner_loop},
    types::{AppState, Args, SimCommand},
    ui::ui_loop,
};

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

    if !owner_done && let Err(e) = owner_handle.await {
        error!("simulation owner task error: {e:?}");
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
