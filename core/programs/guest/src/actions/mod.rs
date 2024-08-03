use mugraph_core::{error::Result, types::*};
use risc0_zkvm::guest::env;

pub fn compose(operations: Vec<Operation>) -> Result<Reaction> {
    let mut nullifiers = Vec::new();

    for operation in operations.iter() {
        verify(operation)?;

        match operation {
            Operation::Consume { input, .. } => {
                nullifiers.push(input.hash()?);
            }
            Operation::Fission { input, .. } => {
                nullifiers.push(input.hash()?);
            }
            Operation::Fusion { inputs, .. } => {
                for input in inputs {
                    nullifiers.push(input.hash()?);
                }
            }
        }
    }

    Ok(Reaction { nullifiers })
}

pub fn verify(operation: &Operation) -> Result<()> {
    match operation {
        Operation::Fission { input, outputs } => {
            assert_ne!(outputs.len(), 0);

            if let Some(program_id) = input.data.program_id {
                env::verify(program_id, &operation.to_bytes()?)
                    .expect("Failed to run input program.");
            }

            let mut output_total = 0;

            for output in outputs {
                assert_eq!(output.asset_id, input.data.asset_id);
                assert_ne!(output.amount, 0);

                output_total += output.amount;
            }

            assert_eq!(input.data.amount, output_total);
        }
        Operation::Fusion { inputs, output } => {
            assert_ne!(inputs.len(), 0);

            let asset_id = inputs[0].data.asset_id;
            assert_eq!(asset_id, output.asset_id);

            let mut input_total = 0;

            for input in inputs {
                assert_eq!(asset_id, input.data.asset_id);
                assert_ne!(0, input.data.amount);

                if let Some(program_id) = input.data.program_id {
                    env::verify(program_id, &operation.to_bytes()?)
                        .expect("Failed to run input program.");
                }

                input_total += input.data.amount;
            }

            assert_eq!(input_total, output.amount);
        }
        Operation::Consume { input, output } => {
            assert_eq!(input.data.asset_id, output.asset_id);
            assert_ne!(0, input.data.amount);
            assert_eq!(input.data.amount, output.amount);

            if let Some(program_id) = input.data.program_id {
                env::verify(program_id, &operation.to_bytes()?)
                    .expect("Failed to run input program.");
            }
        }
    }

    Ok(())
}
