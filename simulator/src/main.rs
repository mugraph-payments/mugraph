use std::{thread, time::Duration};

use color_eyre::eyre::{ErrReport, Result};
use mugraph_simulator::{Config, Simulator};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tokio::{runtime::Builder, select};
use tokio_util::sync::CancellationToken;
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cores = core_affinity::get_core_ids().unwrap();
    let token = CancellationToken::new();
    let config = Config::default();

    for (i, core) in cores.into_iter().enumerate().skip(1).take(config.threads) {
        info!("Starting simulator on core {i}");

        let token = token.clone();
        thread::spawn(move || {
            core_affinity::set_for_current(core);

            let rt = Builder::new_current_thread().enable_all().build()?;
            rt.block_on(async move {
                let mut rng = config.rng();

                let seed = rng.gen();

                info!(
                    seed = seed,
                    "Starting simulator on core {i} with pre-determined seed."
                );
                let rng = ChaCha20Rng::seed_from_u64(seed);

                let mut simulator = Simulator::build(rng, config).await?;

                loop {
                    select! {
                        _ = token.cancelled() => break,
                        t = simulator.tick() => {
                            simulator = t?;
                        }
                    }
                }

                #[allow(unreachable_code)]
                Ok::<_, ErrReport>(())
            })?;

            #[allow(unreachable_code)]
            Ok::<_, ErrReport>(())
        });
    }

    let t = token.clone();
    ctrlc::set_handler(move || t.cancel()).expect("Error setting Ctrl-C handler");

    thread::sleep(Duration::from_secs(
        config.duration_secs.unwrap_or(u64::MAX),
    ));

    token.cancel();
    info!("Simulation reached end of duration, stopping.");

    Ok(())
}
