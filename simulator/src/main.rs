use std::{
    net::SocketAddr,
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::{ErrReport, Result};
use metrics::{counter, describe_counter, describe_histogram};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_simulator::{Config, Simulation};
use tokio_util::sync::CancellationToken;
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
        "mugraph.simulator.time_taken",
        metrics::Unit::Milliseconds,
        "The time taken to process a single transaction"
    );
    describe_counter!(
        "mugraph.simulator.ticks",
        "The number of ticks processed by the simulator"
    );

    let cores = core_affinity::get_core_ids().unwrap();
    let token = CancellationToken::new();
    let config = Config::default();

    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        info!("Starting simulator on core {i}");

        let token = token.clone();
        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {i}.");

            let mut sim = Simulation::new(config)?;

            loop {
                let start = Instant::now();

                if token.is_cancelled() {
                    break;
                }

                sim.tick()?;

                counter!("mugraph.simulator.time_elapsed_ms")
                    .increment(start.elapsed().as_millis().try_into()?);
            }

            #[allow(unreachable_code)]
            Ok::<_, ErrReport>(())
        });
    }

    let t = token.clone();
    ctrlc::set_handler(move || t.cancel()).expect("Error setting Ctrl-C handler");

    let start = Instant::now();

    while !token.is_cancelled() {
        if start.elapsed() > Duration::from_secs(config.duration_secs.unwrap_or(u64::MAX)) {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    token.cancel();
    info!("Simulation reached end of duration, stopping.");

    Ok(())
}
