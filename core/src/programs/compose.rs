use risc0_zkvm::guest::env;

use crate::{error::Result, types::*};

pub fn compose(req: Request<Vec<Operation>>) -> Result<Reaction> {
    let mut nullifiers = Vec::new();

    for operation in req.data.iter() {
        env::verify(req.manifest.programs.apply, operation.id()?.as_ref()).unwrap();

        match operation {
            Operation::UNSAFE_Mint { .. } => {
                // Do nothing.
            }
            Operation::Consume { input, .. } => {
                nullifiers.push(input.hash()?);
            }
            Operation::Split { input, .. } => {
                nullifiers.push(input.hash()?);
            }
            Operation::Join { inputs, .. } => {
                for input in inputs {
                    nullifiers.push(input.hash()?);
                }
            }
        }
    }

    Ok(Reaction { nullifiers })
}
