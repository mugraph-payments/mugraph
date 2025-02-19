#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use mugraph_simulator::{
    tick,
    tui::{Dashboard, DashboardEvent},
    Config, Delegate, Simulation, TOTAL_TRANSACTIONS,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::default();
    let mut rng = match config.seed {
        Some(s) => ChaCha20Rng::seed_from_u64(s),
        None => ChaCha20Rng::from_entropy(),
    };

    let mut cores = VecDeque::from(core_affinity::get_core_ids().unwrap());
    let is_running = Arc::new(AtomicBool::new(false));
    let is_preparing = Arc::new(AtomicBool::new(true));
    let keypair = Keypair::random(&mut rng);

    // Create the dashboard
    let (mut dashboard, dashboard_tx) = Dashboard::new();

    // Force interface to run on the last possible core
    core_affinity::set_for_current(cores.pop_back().unwrap());

    info!("Starting simulation");

    while !cores.is_empty() {
        let core = cores.pop_front().unwrap();
        let ir = is_running.clone();
        let ip = is_preparing.clone();
        let seed: u64 = rng.gen();
        let tx = dashboard_tx.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            let log_msg = format!("Preparing simulation on core {}, seed {}", core.id, seed);
            tx.send(DashboardEvent::Log(log_msg)).ok();

            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let delegate = Delegate::new(&mut rng, keypair)?;
            let mut sim = Simulation::new(&mut rng, core.id as u32, delegate)?;

            while ip.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(25));
            }

            tx.send(DashboardEvent::Log(format!(
                "Starting simulation on core {}",
                core.id
            )))
            .ok();

            // Wait for signal to start the simulation
            for round in 0u64.. {
                if let Err(e) = tick(core.id, &mut sim, round) {
                    ir.store(false, Ordering::SeqCst);

                    tx.send(DashboardEvent::Log(format!(
                        "Error on core {}: {}",
                        core.id, e
                    )))
                    .ok();

                    Err(e)?;
                }
            }

            Ok::<_, Error>(())
        });
    }

    thread::sleep(Duration::from_millis(50));
    is_running.store(true, Ordering::SeqCst);
    is_preparing.store(false, Ordering::SeqCst);
    let start_time = Instant::now();

    dashboard_tx
        .send(DashboardEvent::Log(
            "Signaled all simulations to start.".to_string(),
        ))
        .ok();

    // Add global metrics reporter
    let ir_metrics = is_running.clone();
    let tx_metrics = dashboard_tx.clone();
    thread::spawn(move || {
        while ir_metrics.load(Ordering::Relaxed) {
            let elapsed = start_time.elapsed().as_secs_f64();
            let total = TOTAL_TRANSACTIONS.load(Ordering::Relaxed);
            let tps = if elapsed > 0.0 {
                total as f64 / elapsed
            } else {
                0.0
            };

            tx_metrics
                .send(DashboardEvent::Metrics {
                    elapsed,
                    tps: tps.round() as u64,
                    p50: 0,
                    p90: 0,
                    p99: 0,
                })
                .ok();

            thread::sleep(Duration::from_millis(100));
        }
    });

    // Run the dashboard in a separate thread
    let ir = is_running.clone();
    thread::spawn(move || {
        if let Err(e) = dashboard.run() {
            ir.store(false, Ordering::SeqCst);

            eprintln!("Dashboard error: {}", e);
        }
    });

    while is_running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}
