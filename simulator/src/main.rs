#![feature(duration_millis_float)]

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::{ErrReport, Result};
use metrics::{describe_histogram, Unit};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::error::Error;
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
        "mugraph.database.len.time_taken",
        Unit::Milliseconds,
        "database time call #len"
    );
    describe_histogram!(
        "mugraph.database.read.time_taken",
        Unit::Milliseconds,
        "database time call #read"
    );
    describe_histogram!(
        "mugraph.database.set_len.time_taken",
        Unit::Milliseconds,
        "database time call #set_len"
    );
    describe_histogram!(
        "mugraph.database.sync_data.time_taken",
        Unit::Milliseconds,
        "database time call #sync_data"
    );
    describe_histogram!(
        "mugraph.database.write.time_taken",
        Unit::Milliseconds,
        "database time call #write"
    );
    describe_histogram!(
        "mugraph.simulator.tick.time_taken",
        Unit::Milliseconds,
        "how long it took to run a simulation tick"
    );
    describe_histogram!(
        "mugraph.simulator.state.next.time_taken",
        Unit::Milliseconds,
        "how long it took to generate the next action in the simulation"
    );
    describe_histogram!(
        "mugraph.simulator.state.next.split.time_taken",
        Unit::Milliseconds,
        "how long it took to generate the next split action in the simulation"
    );
    describe_histogram!(
        "mugraph.simulator.state.next.join.time_taken",
        Unit::Milliseconds,
        "how long it took to generate the next join action in the simulation"
    );
    describe_histogram!(
        "mugraph.simulator.delegate.transaction_v0",
        Unit::Milliseconds,
        "How long it took to get a server response"
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

    metrics_observer::main(should_continue.clone()).map_err(|e| Error::ServerError {
        reason: e.to_string(),
    })?;

    should_continue.swap(false, Ordering::Relaxed);

    thread::sleep(Duration::from_millis(100));

    info!("Simulation reached end of duration, stopping.");

    Ok(())
}
