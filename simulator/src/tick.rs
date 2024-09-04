use std::{
    backtrace::Backtrace,
    panic::{self, AssertUnwindSafe},
};

use mugraph_core::error::Error;
use tracing::error;

use crate::Simulation;

#[tracing::instrument(skip(sim))]
pub fn tick(core_id: usize, sim: &mut Simulation, round: u64) -> Result<(), Error> {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        sim.tick(round)?;
        Ok::<_, Error>(())
    }));

    match result {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            let bt = Backtrace::capture().to_string();
            error!(reason = %e, backtrace = %bt, "Simulation errored.");

            Err(e)?;
        }
        Err(e) => {
            let bt = Backtrace::capture().to_string();

            if let Some(message) = e.downcast_ref::<&str>() {
                error!(reason = message, backtrace = %bt, "Simulation panicked!");
            } else if let Ok(message) = e.downcast::<String>() {
                error!(reason = message, backtrace = %bt, "Simulation panicked!");
            } else {
                error!(reason = "Could not retrieve", backtrace = %bt, "Simulation panicked!");
            }

            return Err(Error::SimulationError {
                reason: "Simulation panicked".to_string(),
            });
        }
    }

    Ok(())
}
