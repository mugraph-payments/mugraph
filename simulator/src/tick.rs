use std::backtrace::Backtrace;

use mugraph_core::error::Error;
use tracing::error;

use crate::Simulation;

#[tracing::instrument(skip(sim))]
pub fn tick(
    core_id: usize,
    sim: &mut Simulation,
    round: u64,
) -> Result<(), Error> {
    match sim.tick(round) {
        Ok(_) => {}
        Err(e) => {
            let bt = Backtrace::capture().to_string();
            error!(reason = %e, backtrace = %bt, "Simulation errored.");

            Err(e)?;
        }
    }

    Ok(())
}
