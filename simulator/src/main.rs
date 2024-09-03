#![feature(duration_millis_float)]

use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::{ErrReport, Result};
use metrics::{describe_histogram, gauge, Unit};
use metrics_exporter_tcp::TcpBuilder;
use metrics_observer::Client;
use mugraph_core::types::Keypair;
use mugraph_simulator::{Config, Delegate, Simulation};
use tracing::{error, info};

fn main() -> Result<()> {
    color_eyre::install()?;
    let metric_address = "0.0.0.0:9999";
    TcpBuilder::new()
        .listen_address(metric_address.parse::<SocketAddr>()?)
        .install()?;
    tracing_subscriber::fmt().init();

    let mut cores = VecDeque::from(core_affinity::get_core_ids().unwrap());
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();
    let observer_client = Client::new(metric_address.to_string());
    let mut rng = config.rng();

    describe_histogram!(
        "metrics-observer.frame_time",
        Unit::Milliseconds,
        "How long it takes to render a frame of the observer"
    );
    describe_histogram!(
        "transaction.verify",
        Unit::Milliseconds,
        "How long it takes to verify a transaction"
    );
    describe_histogram!(
        "database.len",
        Unit::Milliseconds,
        "database time call #len"
    );
    describe_histogram!(
        "database.read",
        Unit::Milliseconds,
        "database time call #read"
    );
    describe_histogram!(
        "database.set_len",
        Unit::Milliseconds,
        "database time call #set_len"
    );
    describe_histogram!(
        "database.sync_data",
        Unit::Milliseconds,
        "database time call #sync_data"
    );
    describe_histogram!(
        "database.write",
        Unit::Milliseconds,
        "database time call #write"
    );
    describe_histogram!(
        "tick_time",
        Unit::Milliseconds,
        "how long it took to run a simulation tick"
    );
    describe_histogram!(
        "state.next",
        Unit::Milliseconds,
        "how long it took to generate the next action in the simulation"
    );
    describe_histogram!(
        "state.next.split",
        Unit::Milliseconds,
        "how long it took to generate the next split action in the simulation"
    );
    describe_histogram!(
        "state.next.join",
        Unit::Milliseconds,
        "how long it took to generate the next join action in the simulation"
    );
    describe_histogram!(
        "delegate.transaction_v0",
        Unit::Milliseconds,
        "How long it took to get a server response"
    );

    // Force interface to run on the first possible core
    core_affinity::set_for_current(cores.pop_front().unwrap());
    let threads = config.threads - 1;
    info!(count = threads, "Starting simulations");
    let keypair = Keypair::random(&mut rng);
    let delegate = Delegate::new(&mut rng, keypair)?;
    let simulations = (1..=threads)
        .map(|i| Simulation::new(i as u32, delegate.clone()))
        .filter_map(|x| x.ok());

    for (core, mut sim) in cores.into_iter().zip(simulations).take(threads) {
        let sc = should_continue.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {}.", core.id);

            let mut round = 0;

            while sc.load(Ordering::Relaxed) {
                gauge!("current_round", "core_id" => core.id.to_string()).set(round as f64);

                sim.tick(round)?;
                round += 1;
            }

            Ok::<_, ErrReport>(())
        });
    }

    thread::sleep(Duration::from_millis(100));

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
