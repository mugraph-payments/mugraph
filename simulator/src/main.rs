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
use itertools::Itertools;
use metrics::{describe_histogram, gauge, Unit};
use metrics_exporter_tcp::TcpBuilder;
use metrics_observer::Client;
use mugraph_simulator::{Config, Simulation};
use tracing::{error, info};

fn main() -> Result<()> {
    color_eyre::install()?;
    let metric_address = "0.0.0.0:9999";
    TcpBuilder::new()
        .listen_address(metric_address.parse::<SocketAddr>()?)
        .install()?;
    tracing_subscriber::fmt().init();

    let mut cores = core_affinity::get_core_ids().unwrap();
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();
    let observer_client = Client::new(metric_address.to_string());

    describe_histogram!(
        "metrics-observer.frame_time",
        Unit::Milliseconds,
        "How long it takes to render a frame of the observer"
    );
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

    // Force interface to run on the first possible core
    core_affinity::set_for_current(cores.pop().unwrap());
    let threads = config.threads - 1;
    info!(count = threads, "Starting simulations");
    let simulations: Vec<Simulation> = (0..threads)
        .map(|i| Simulation::new(i as u32))
        .try_collect()?;

    for (core, mut sim) in cores.into_iter().zip(simulations).take(threads) {
        let sc = should_continue.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {}.", core.id);

            let mut round = 0;

            while sc.load(Ordering::Relaxed) {
                gauge!(
                    "mugraph.simulator.current_round",
                    "core_id" => core.id.to_string()
                )
                .set(round as f64);

                sim.tick(round)?;
                round += 1;
            }

            Ok::<_, ErrReport>(())
        });
    }

    thread::sleep(Duration::from_millis(100));

    let sc = should_continue.clone();
    ctrlc::set_handler(move || {
        sc.swap(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    match metrics_observer::main(observer_client, should_continue.clone()) {
        Ok(_) => {
            info!("Observer finished.");
        }
        Err(e) => {
            error!(msg = %e, "Observer failed because of error");
        }
    }

    should_continue.swap(false, Ordering::Relaxed);
    metrics_observer::restore_terminal()?;

    Ok(())
}
