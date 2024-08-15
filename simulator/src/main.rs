use std::{
    future::ready,
    thread::{self, JoinHandle},
};

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

    for (i, core) in cores.into_iter().enumerate().take(num_cpus::get_physical()) {
        let span = span!(tracing::Level::INFO, "simulator");
        span.record("core", i);

        let _ = span.enter();

        let t = token.clone();
        let handle: JoinHandle<Result<(), ErrReport>> = thread::spawn(move || {
            core_affinity::set_for_current(core);

            let rt = Builder::new_current_thread().enable_all().build()?;
            rt.block_on(async move {
                let config = Config::default();
                let mut simulator = Simulator::build(config).await?;

                loop {
                    select! {
                        _ = t.cancelled() => break,
                        _ = simulator.tick() => {
                            info!("Processed tick...");
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

    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {
                info!("Thread finished");
            }
            Ok(Err(e)) => {
                error!("Thread failed: {:?}", e);
            }
            Err(e) => {
                error!("Thread failed: {:?}", e);
            }
        }

        info!("Cancelling all other tasks");
        token.cancel();
    }

    #[allow(unreachable_code)]
    Ok(())
}
