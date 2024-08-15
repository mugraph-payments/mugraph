use std::thread::{self, JoinHandle};

use color_eyre::eyre::{ErrReport, Result};
use mugraph_simulator::{Config, Simulator};
use tokio::runtime::Builder;
use tracing::span;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cores = core_affinity::get_core_ids().unwrap();
    let mut handles = vec![];

    for (i, core) in cores.into_iter().enumerate().take(num_cpus::get_physical()) {
        let span = span!(tracing::Level::INFO, "simulator");
        span.record("core", i);

        let _ = span.enter();

        let handle: JoinHandle<Result<(), ErrReport>> = thread::spawn(move || {
            core_affinity::set_for_current(core);

            let rt = Builder::new_current_thread().enable_all().build()?;
            rt.block_on(async move {
                let config = Config::default();
                let mut simulator = Simulator::new().setup(config).await?;

                loop {
                    simulator.tick().await?;
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
            Ok(Ok(())) => continue,
            Ok(Err(e)) => {
                eprintln!("Thread failed: {:?}", e);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Thread failed: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
