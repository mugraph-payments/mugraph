use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::{ErrReport, Result};
use metrics::{describe_counter, describe_histogram};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_simulator::{Config, Simulation};
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    TcpBuilder::new()
        .listen_address("0.0.0.0:9999".parse::<SocketAddr>()?)
        .install()?;

    describe_counter!(
        "mugraph.simulator.processed_transactions",
        "The number of processed transactions during the simulation"
    );
    describe_histogram!(
        "mugraph.simulator.time_elapsed",
        metrics::Unit::Milliseconds,
        "The time taken to process a single transaction"
    );
    describe_counter!(
        "mugraph.node.database.len_calls",
        "Number of calls to storage's #len"
    );
    describe_counter!(
        "mugraph.node.database.set_len_calls",
        "Number of calls to storage's #set_len"
    );
    describe_counter!(
        "mugraph.node.database.sync_data_calls",
        "Number of calls to storage's #sync_data"
    );
    describe_counter!(
        "mugraph.node.database.write_calls",
        "Number of calls to storage's #write"
    );
    describe_counter!(
        "mugraph.node.database.injected_failures",
        "Number of storage failures injected"
    );

    let cores = core_affinity::get_core_ids().unwrap();
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();

    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        info!("Starting simulator on core {i}");

        let sc = should_continue.clone();
        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {i}.");

            let mut sim = Simulation::new(config)?;

            while sc.load(Ordering::Relaxed) {
                sim.tick()?;
            }

            #[allow(unreachable_code)]
            Ok::<_, ErrReport>(())
        });
    }

    let sc = should_continue.clone();
    ctrlc::set_handler(move || {
        sc.swap(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let start = Instant::now();

    while should_continue.load(Ordering::Relaxed) {
        if start.elapsed() > Duration::from_secs(config.duration_secs.unwrap_or(u64::MAX)) {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    should_continue.swap(false, Ordering::Relaxed);
    info!("Simulation reached end of duration, stopping.");

    Ok(())
}
