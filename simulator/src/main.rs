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
    types::{AppState, Args, SimChannels, SimCommand, SimConfig, SimNode},
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

    let mut nodes = Vec::new();
    for url in &args.node_urls {
        let client = NodeClient::new(url)?;
        client
            .health()
            .await
            .wrap_err_with(|| format!("health check failed for {url}"))?;
        let delegate_pk = client
            .public_key()
            .await
            .wrap_err_with(|| format!("fetch public key from {url}"))?;
        info!("connected to node {url} (delegate pk {delegate_pk})");
        nodes.push(SimNode {
            client,
            delegate_pk,
        });
    }

    if nodes.len() > 1 {
        info!(
            "{} nodes connected — cross-node transfers will be exercised",
            nodes.len()
        );
    }

    let delegates: Vec<_> = nodes.iter().map(|n| n.delegate_pk).collect();

    let assets = generate_assets(args.assets, &mut rng);
    let mut state = AppState {
        assets: assets.clone(),
        delegates,
        ..Default::default()
    };

    bootstrap_wallets(
        &nodes,
        &mut state,
        &assets,
        args.wallets,
        args.notes_per_wallet,
        (args.min_amount, args.max_amount),
        &mut rng,
    )
    .await
    .wrap_err("bootstrap wallets")?;

    let config = SimConfig {
        amount_range: (args.min_amount, args.max_amount),
        tick: Duration::from_millis(args.tick_ms),
        max_inflight: args.max_inflight,
    };

    let (snapshot_tx, snapshot_rx) =
        watch::channel(state.snapshot(0, args.max_inflight, 0.0, 100.0));
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let channels = SimChannels {
        cmd_rx,
        event_rx,
        event_tx,
        snapshot_tx,
    };

    let mut owner_handle = tokio::spawn(simulation_owner_loop(
        nodes, state, rng, config, channels,
    ));

    let terminal = ratatui::init();
    let ui_cmd_tx = cmd_tx.clone();
    let mut ui_handle = tokio::task::spawn_blocking(move || {
        ui_loop(snapshot_rx, ui_cmd_tx, terminal)
    });

    let mut owner_completed = false;
    let mut ui_completed = false;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl+c, shutting down");
        }
        res = &mut owner_handle => {
            owner_completed = true;
            if let Err(e) = res {
                error!("simulation owner task error: {e:?}");
            }
        }
        res = &mut ui_handle => {
            ui_completed = true;
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => error!("ui task error: {e:#}"),
                Err(e) => error!("ui task join error: {e:?}"),
            }
        }
    }

    // Signal shutdown — sending to a closed channel is a no-op
    let _ = cmd_tx.send(SimCommand::Quit);

    if !owner_completed {
        let _ = owner_handle.await;
    }
    if !ui_completed {
        match ui_handle.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => error!("ui task error: {e:#}"),
            Err(e) => error!("ui task join error: {e:?}"),
        }
    }

    ratatui::restore();
    Ok(())
}
