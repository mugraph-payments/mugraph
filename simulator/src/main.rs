#![feature(duration_millis_float)]

use std::{
    collections::VecDeque,
    net::SocketAddr,
    panic::{self, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::Result;
use metrics::{describe_histogram, gauge, Unit};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::{error::Error, types::Keypair};
use mugraph_simulator::{
    observer::{self, Client},
    Config, Delegate, Simulation,
};
use tracing::{error, info};

fn main() -> Result<()> {
    color_eyre::install()?;
    let metric_address = "0.0.0.0:9999";
    TcpBuilder::new()
        .listen_address(metric_address.parse::<SocketAddr>()?)
        .install()?;
    tracing_subscriber::fmt().init();

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
    describe_histogram!(
        "observer.frame_time",
        Unit::Milliseconds,
        "How long it takes to render a frame of the observer"
    );

    let mut cores = VecDeque::from(core_affinity::get_core_ids().unwrap());
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();
    let mut rng = config.rng();
    let threads = config.threads - 1;

    let keypair = Keypair::random(&mut rng);
    let delegate = Delegate::new(&mut rng, keypair)?;
    let observer_client = Client::new(metric_address.to_string());

    // Force interface to run on the first possible core
    core_affinity::set_for_current(cores.pop_front().unwrap());

    info!(count = threads, "Starting simulations");
    let simulations = (1..=threads)
        .map(|i| Simulation::new(i as u32, delegate.clone()))
        .filter_map(|x| x.ok());

    for (core, mut sim) in cores.into_iter().zip(simulations).take(threads) {
        let sc = should_continue.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {}.", core.id);

            let mut round = 0;

            let core_id = core.id.to_string();

            while sc.load(Ordering::Relaxed) {
                gauge!("current_round", "core_id" => core_id.clone()).set(round as f64);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    sim.tick(round)?;

                    Ok::<_, Error>(())
                }));

                match result {
                    Ok(_) => {}
                    Err(err) => {
                        observer::restore_terminal()?;

                        if let Some(message) = err.downcast_ref::<&str>() {
                            error!(message = message, "Simulation panicked!");
                        } else if let Ok(message) = err.downcast::<String>() {
                            error!(message = message, "Simulation panicked!");
                        } else {
                            error!(message = "Could not retrieve", "Simulation panicked!");
                        }

                        sc.store(false, Ordering::SeqCst);
                    }
                }

                round += 1;
            }

            Ok::<_, Error>(())
        });
    }

    thread::sleep(Duration::from_millis(100));

    match observer::main(observer_client, should_continue.clone()) {
        Ok(_) => {
            observer::restore_terminal()?;
            info!("Observer finished.");
        }
        Err(e) => {
            observer::restore_terminal()?;
            error!(msg = %e, "Observer failed because of error");
        }
    }

    should_continue.swap(false, Ordering::Relaxed);

    Ok(())
}
