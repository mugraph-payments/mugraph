use std::thread::{self, JoinHandle};

use color_eyre::eyre::{ErrReport, Result};
use mugraph_simulator::{Config, Simulator};
use tokio::{runtime::Builder, select};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, span};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cores = core_affinity::get_core_ids().unwrap();
    let mut handles = vec![];
    let token = CancellationToken::new();

    let t = token.clone();
    ctrlc::set_handler(move || t.cancel()).expect("Error setting Ctrl-C handler");

    let config = Config::default();

    for (i, core) in cores.into_iter().enumerate().take(config.threads) {
        let span = span!(tracing::Level::INFO, "simulator");
        span.record("core", i);

        let _ = span.enter();

        let t = token.clone();
        let handle: JoinHandle<Result<(), ErrReport>> = thread::spawn(move || {
            core_affinity::set_for_current(core);

            let rt = Builder::new_current_thread().enable_all().build()?;
            rt.block_on(async move {
                let mut simulator = Simulator::build(config).await?;

                loop {
                    let mut i = 1;

                    select! {
                        _ = t.cancelled() => break,
                        _ = simulator.tick() => {
                            i += 1;

                            if config.steps.map(|s| i >= s).unwrap_or(false) {
                                info!("Reached end of simulation");
                                t.cancel();
                            }
                        }
                    }
                }

                #[allow(unreachable_code)]
                Ok::<_, ErrReport>(())
            })?;

            #[allow(unreachable_code)]
            Ok::<_, ErrReport>(())
        });

        handles.push(handle);
    }

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                info!("Thread {i} finished");
            }
            Ok(Err(e)) => {
                error!("Thread {i} failed: {:?}", e);
            }
            Err(e) => {
                error!("Thread {i} failed: {:?}", e);
            }
        }

        token.cancel();
        break;
    }

    #[allow(unreachable_code)]
    Ok(())
}
