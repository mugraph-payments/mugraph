#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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
use mugraph_simulator::{observer, tick, Config, Delegate, Simulation};
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
    let is_running = Arc::new(AtomicBool::new(false));
    let config = Config::default();
    let last_core = cores.pop_back().unwrap();

    // Force interface to run on the last possible core
    core_affinity::set_for_current(last_core);

    info!("Starting simulation");

    let core = cores.pop_front().unwrap();

    let ir = is_running.clone();
    thread::spawn(move || {
        info!(core_id = core.id, "Starting simulation");

        while !ir.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }

        core_affinity::set_for_current(core);

        let mut rng = config.rng();
        let keypair = Keypair::random(&mut rng);
        let delegate = Delegate::new(&mut rng, keypair)?;
        let mut sim = Simulation::new(&mut rng, core.id as u32, delegate)?;

        // Wait for signal to start the simulation
        for round in 0u64.. {
            if let Err(e) = tick(core.id, &mut sim, round) {
                ir.store(false, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(50));

                Err(e)?;
            }
        }

        Ok::<_, Error>(())
    });

    is_running.store(true, Ordering::SeqCst);

    match observer::main(&is_running) {
        Ok(_) => {
            observer::restore_terminal()?;
            info!("Observer finished.");
        }
        Err(e) => {
            observer::restore_terminal()?;
            error!(msg = %e, "Observer failed because of error");
        }
    }

    Ok(())
}
