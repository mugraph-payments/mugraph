use crate::{error::Error, types::*};

pub struct Proof {}

/// Generates a Zero-Knowledge Proof for a Swap.
///
/// This method should prove:
///
/// - Per asset id, sum of amounts in inputs equal sum of amounts in outputs.
/// - All the asset ids in the inputs are present in the outputs.
/// - There are no outputs with amount 0.
pub fn prove_swap(inputs: &[Note], outputs: &[UnblindedNote]) -> Result<Proof, Error> {
    todo!("Please implement this according to the specifications.");
}

/// Verifies a Zero-Knowledge Proof for a Swap.
pub fn verify_swap_proof(swap: &Swap, proof: &Proof) -> Result<(), Error> {
    todo!();
}
