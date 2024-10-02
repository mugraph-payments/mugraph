#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use std::{
    collections::VecDeque,
    net::SocketAddr,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use mugraph_node;
use mugraph_simulator::{tick, Config, Delegate, Simulation};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tokio::runtime::Runtime;
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

    let node_core = cores.pop_front().unwrap();
    let node_seed = rng.gen();

    let protocol_end = config.node_endpoint.rfind("/").unwrap_or(0) + 1; // FIXME: dont add if the value is already 0
    let node_endpoint = config.node_endpoint[protocol_end..].to_string();

    thread::spawn(move || {
        core_affinity::set_for_current(node_core);
        info!(core_id = node_core.id, seed = node_seed, public_key = %keypair.public_key,  "Starting node");

        let config = mugraph_node::config::Config {
            addr: SocketAddr::from_str(&node_endpoint).expect("Invalid address"),
            seed: node_seed,
            public_key: Some(
                serde_json::to_string(&keypair.public_key).expect("Failed to serialize public key"),
            ),
            secret_key: Some(
                serde_json::to_string(&keypair.secret_key).expect("Failed to serialize secret key"),
            ),
        };

        let rt = Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            if let Err(e) = mugraph_node::start(&config).await {
                eprintln!("Failed to start server: {:?}", e);
            }
        });

        Ok::<_, Error>(())
    });

    info!("Starting simulation");

    while !cores.is_empty() {
        let core = cores.pop_front().unwrap();
        let ir = is_running.clone();
        let ip = is_preparing.clone();
        let seed: u64 = rng.gen();
        let node_target = config.node_endpoint.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!(core_id = core.id, seed = seed, "Preparing simulation");

            let mut rng = ChaCha20Rng::seed_from_u64(seed);

            let delegate = Delegate::new(&mut rng, keypair, node_target)?;
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
