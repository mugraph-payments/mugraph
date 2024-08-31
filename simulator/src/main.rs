use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::{ErrReport, Result};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::types::Hash;
use mugraph_node::context::Context;
use mugraph_simulator::{Config, Delegate, Simulation};
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
    let mut rng = config.rng();

    let context = Context::new(&mut rng)?;
    let delegate = Delegate::new(&mut rng, config.node_url, context)?;
    let assets = (0..config.assets)
        .map(|_| Hash::random(&mut rng))
        .collect::<Vec<_>>();

    let users = Arc::new(RwLock::new(vec![]));
    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        info!("Starting simulator on core {i}");

        let sc = should_continue.clone();
        let d = delegate.clone();
        let assets = assets.clone();
        let mut sim = Simulation::new(core.id as u32, config, d, assets, users.clone())?;

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {i}.");

            let mut round = 0;
            while sc.load(Ordering::Relaxed) {
                round += 1;

                sim.tick(round)?;
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
