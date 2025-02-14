#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use mugraph_simulator::{tick, Config, Delegate, Simulation};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let config = Config::default();
    let mut rng = match config.seed {
        Some(s) => ChaCha20Rng::seed_from_u64(s),
        None => ChaCha20Rng::from_entropy(),
    };

    let mut cores = VecDeque::from(core_affinity::get_core_ids().unwrap());
    let is_running = Arc::new(AtomicBool::new(false));
    let is_preparing = Arc::new(AtomicBool::new(true));
    let keypair = Keypair::random(&mut rng);

    // Force interface to run on the last possible core
    core_affinity::set_for_current(cores.pop_back().unwrap());

    info!("Starting simulation");

    while !cores.is_empty() {
        let core = cores.pop_front().unwrap();
        let ir = is_running.clone();
        let ip = is_preparing.clone();
        let seed: u64 = rng.gen();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!(core_id = core.id, seed = seed, "Preparing simulation");

            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let delegate = Delegate::new(&mut rng, keypair)?;
            let mut sim = Simulation::new(&mut rng, core.id as u32, delegate)?;

            while ip.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(25));
            }

            info!(core_id = core.id, seed = seed, "Starting simulation");

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
    }

    thread::sleep(Duration::from_millis(50));
    is_running.store(true, Ordering::SeqCst);
    is_preparing.store(false, Ordering::SeqCst);

    info!("Signaled all simulations to start.");

    while is_running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}
