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
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::{error::Error, types::Keypair, utils::describe_metrics};
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

    describe_metrics();

    let mut cores = VecDeque::from(core_affinity::get_core_ids().unwrap());
    let is_running = Arc::new(AtomicBool::new(true));
    let config = Config::default();
    let mut rng = config.rng();
    let threads = config.threads - 1;

    let keypair = Keypair::random(&mut rng);
    let delegate = Delegate::new(&mut rng, keypair)?;
    let last_core = cores.pop_back().unwrap();
    let observer_client = Client::new(last_core, metric_address.to_string());

    // Force interface to run on the last possible core
    core_affinity::set_for_current(last_core);

    info!(count = threads, "Starting simulations");
    let simulations = (0..threads)
        .map(|i| Simulation::new(&mut rng, i as u32, delegate.clone()))
        .filter_map(|x| x.ok());

    for (core, mut sim) in cores.into_iter().zip(simulations).take(threads) {
        let is_running = is_running.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!(core_id = core.id, "Starting simulation");

            // Wait for signal to start the simulation
            info!(core_id = core.id, "Waiting for signal to start simulation");
            if !is_running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(50));
            }

            for round in 0u64.. {
                if let Err(e) = tick(core.id, &mut sim, round) {
                    is_running.store(false, Ordering::SeqCst);
                    Err(e)?;
                }
            }

            Ok::<_, Error>(())
        });
    }

    thread::sleep(Duration::from_millis(100));
    is_running.store(true, Ordering::SeqCst);

    match observer::main(observer_client, &is_running) {
        Ok(_) => {
            observer::restore_terminal()?;
            info!("Observer finished.");
        }
        Err(e) => {
            observer::restore_terminal()?;
            error!(msg = %e, "Observer failed because of error");
        }
    }

    is_running.swap(true, Ordering::Relaxed);

    Ok(())
}
