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
    tui::{Dashboard, DashboardEvent, DashboardFormatter},
    Config,
    Delegate,
    Simulation,
    TOTAL_TRANSACTIONS,
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::{error, info};
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Set up the dashboard formatter
    let (formatter, logs) = DashboardFormatter::new();

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().event_format(formatter))
        .init();

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
    let (mut dashboard, dashboard_tx) = Dashboard::new(logs);

    // Force interface to run on the last possible core
    core_affinity::set_for_current(cores.pop_back().unwrap());

    info!("Starting simulation");

    while !cores.is_empty() {
        let core = cores.pop_front().unwrap();
        let ir = is_running.clone();
        let ip = is_preparing.clone();
        let seed: u64 = rng.gen();
        let addr = config.node_addr.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Preparing simulation on core {}, seed {}", core.id, seed);

            loop {
                match (|| -> Result<_, Error> {
                    let mut rng = ChaCha20Rng::seed_from_u64(seed);
                    let delegate = Delegate::new(&mut rng, keypair, &addr)?;
                    let mut sim = Simulation::new(&mut rng, core.id as u32, delegate)?;

                    while ip.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(25));
                    }

                    info!("Starting simulation on core {}", core.id);

                    // Wait for signal to start the simulation
                    while ir.load(Ordering::Relaxed) {
                        for round in 0u64.. {
                            if let Err(e) = tick(core.id, &mut sim, round) {
                                error!("Error on core {}: {}", core.id, e);

                                // Only exit the thread if it's a fatal error
                                if !matches!(e, Error::SimulatedError { .. }) {
                                    return Err(e);
                                }
                            }
                        }
                    }
                    Ok(())
                })() {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Fatal error on core {}: {}. Restarting simulation...",
                            core.id, e
                        );
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                }
            }
        });
    }

    thread::sleep(Duration::from_millis(50));
    is_running.store(true, Ordering::SeqCst);
    is_preparing.store(false, Ordering::SeqCst);
    let start_time = Instant::now();

    info!("Signaled all simulations to start.");

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
            error!("Dashboard error: {}", e);
        }
    });

    // Wait for dashboard thread to signal quit
    while is_running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    // Give threads time to clean up
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
