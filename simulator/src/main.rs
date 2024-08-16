use std::{thread, time::Duration};

use color_eyre::eyre::{ErrReport, Result};
use mugraph_simulator::{Config, Simulator};
use tokio::{runtime::Builder, select};
use tokio_util::sync::CancellationToken;
use tracing::{info, span};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cores = core_affinity::get_core_ids().unwrap();
    let token = CancellationToken::new();
    let config = Config::default();

    for (i, core) in cores.into_iter().enumerate().take(config.threads) {
        let span = span!(tracing::Level::INFO, "simulator");
        span.record("core", i);

        let _ = span.enter();

        let token = token.clone();
        thread::spawn(move || {
            core_affinity::set_for_current(core);

            let rt = Builder::new_current_thread().enable_all().build()?;
            rt.block_on(async move {
                let mut simulator = Simulator::build(config).await?;

                loop {
                    select! {
                        _ = token.cancelled() => break,
                        _ = simulator.tick() => { }
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
