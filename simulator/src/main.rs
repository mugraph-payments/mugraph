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

use color_eyre::eyre::Result;
use metrics::{describe_histogram, Unit};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::{error::Error, types::Keypair};
use mugraph_simulator::{
    observer::{self, Client},
    tick, Config, Delegate, Simulation,
};
use tracing::{error, info};

fn main() -> Result<()> {
    color_eyre::install()?;
    let metric_address = "0.0.0.0:9999";
    TcpBuilder::new()
        .listen_address(metric_address.parse::<SocketAddr>()?)
        .install()?;
    tracing_subscriber::fmt().init();

    describe_histogram!("mugraph.task", Unit::Milliseconds, "Duration of a task");

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

            while sc.load(Ordering::Relaxed) {
                match tick(core.id, &mut sim, round) {
                    Ok(_) => {
                        round += 1;
                    }
                    e => {
                        sc.store(false, Ordering::SeqCst);
                        e?;
                    }
                }
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
