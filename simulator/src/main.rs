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
use metrics::{describe_counter, describe_histogram};
use metrics_exporter_tcp::TcpBuilder;
use mugraph_core::types::Hash;
use mugraph_node::context::Context;
use mugraph_simulator::{Config, Delegate, Simulation, User};
use rand::Rng;
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    TcpBuilder::new()
        .listen_address("0.0.0.0:9999".parse::<SocketAddr>()?)
        .install()?;

    describe_counter!(
        "mugraph.simulator.processed_transactions",
        "The number of processed transactions during the simulation"
    );
    describe_histogram!(
        "mugraph.simulator.time_elapsed",
        metrics::Unit::Milliseconds,
        "The time taken to process a single transaction"
    );
    describe_counter!(
        "mugraph.node.database.len_calls",
        "Number of calls to storage's #len"
    );
    describe_counter!(
        "mugraph.node.database.set_len_calls",
        "Number of calls to storage's #set_len"
    );
    describe_counter!(
        "mugraph.node.database.sync_data_calls",
        "Number of calls to storage's #sync_data"
    );
    describe_counter!(
        "mugraph.node.database.write_calls",
        "Number of calls to storage's #write"
    );
    describe_counter!(
        "mugraph.node.database.injected_failures",
        "Number of storage failures injected"
    );

    let cores = core_affinity::get_core_ids().unwrap();
    let should_continue = Arc::new(AtomicBool::new(true));
    let config = Config::default();
    let mut rng = config.rng();

    let context = Context::new(&mut rng)?;
    let mut delegate = Delegate::new(&mut rng, context)?;
    let assets = (0..config.assets)
        .map(|_| Hash::random(&mut rng))
        .collect::<Vec<_>>();

    let mut users = vec![];

    for i in 0..config.users {
        let mut notes = vec![];

        for _ in 0..rng.gen_range(1..config.notes) {
            let idx = rng.gen_range(0..config.assets);

            let asset_id = assets[idx];
            let amount = rng.gen_range(1..1_000_000_000);

            let note = delegate.emit(asset_id, amount)?;

            notes.push(note);
        }

        assert_ne!(notes.len(), 0);
        let var_name = User::new();
        let mut user = var_name;
        user.notes = notes;
        users.push(user);

        info!(
            user_id = i,
            notes = users[i].notes.len(),
            "Created notes for user"
        );
    }

    let users = Arc::new(RwLock::new(users));

    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        info!("Starting simulator on core {i}");

        let sc = should_continue.clone();
        let d = delegate.clone();
        let u = users.clone();

        thread::spawn(move || {
            core_affinity::set_for_current(core);

            info!("Starting simulation on core {i}.");

            let mut sim = Simulation::new(core.id as u32, config, d, u)?;

            while sc.load(Ordering::Relaxed) {
                sim.tick()?;
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
