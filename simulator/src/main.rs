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
use metrics::{describe_histogram, Unit};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_simulator::{Config, Simulation};
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    TcpBuilder::new()
        .listen_address("0.0.0.0:9999".parse::<SocketAddr>()?)
        .install()?;

    let cores = core_affinity::get_core_ids().unwrap();
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();

    describe_histogram!(
        "mugraph.node.database.backend_times.len",
        Unit::Milliseconds,
        "database time call #len"
    );
    describe_histogram!(
        "mugraph.node.database.backend_times.read",
        Unit::Milliseconds,
        "database time call #read"
    );
    describe_histogram!(
        "mugraph.node.database.backend_times.set_len",
        Unit::Milliseconds,
        "database time call #set_len"
    );
    describe_histogram!(
        "mugraph.node.database.backend_times.sync_data",
        Unit::Milliseconds,
        "database time call #sync_data"
    );
    describe_histogram!(
        "mugraph.node.database.backend_times.write",
        Unit::Milliseconds,
        "database time call #write"
    );

    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        let sc = should_continue.clone();
        let mut sim = Simulation::new(core.id as u32)?;

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {i}.");

            let mut round = 0;

            while sc.load(Ordering::Relaxed) {
                sim.tick(round)?;
                round += 1;
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
