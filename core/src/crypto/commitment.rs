use std::collections::HashMap;

use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::crypto::*;
use crate::error::Error;
use crate::types::op::swap::{Atom, Swap};
use crate::Hash;

const MAX_INPUTS: usize = 16;
const MAX_OUTPUTS: usize = 16;

#[derive(Debug, Clone)]
pub struct TransactionCommitment {
    pub bulletproof: RangeProof,
    pub asset_commitments: HashMap<CompressedRistretto, CompressedRistretto>,
}

#[cfg(test)]
impl proptest::arbitrary::Arbitrary for TransactionCommitment {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::{collection::vec, prelude::*};

        (vec(any::<Atom>(), 1..=8), vec(any::<Atom>(), 1..=8))
            .prop_map(|(inputs, outputs)| commit(&inputs, &outputs).unwrap())
            .boxed()
    }
}

fn create_transcript(label: &'static [u8]) -> Transcript {
    let mut transcript = Transcript::new(label);
    transcript.append_message(b"protocol-name", COMMITMENT_TRANSCRIPT_LABEL);
    transcript
}

pub fn commit(inputs: &[Atom], outputs: &[Atom]) -> Result<TransactionCommitment, Error> {
    if inputs.len() > MAX_INPUTS {
        return Err(Error::TooManyInputs(inputs.len()));
    }

    if outputs.len() > MAX_OUTPUTS {
        return Err(Error::TooManyOutputs(outputs.len()));
    }

    let mut amounts = Vec::new();
    let mut blindings = Vec::new();
    let mut asset_balances: HashMap<Hash, i128> = HashMap::new();
    let mut asset_commitments: HashMap<CompressedRistretto, CompressedRistretto> = HashMap::new();

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, MAX_INPUTS + MAX_OUTPUTS);

    // Process inputs and outputs
    for (is_input, atom) in inputs
        .iter()
        .map(|a| (true, a))
        .chain(outputs.iter().map(|a| (false, a)))
    {
        if atom.amount == 0 {
            return Err(Error::ZeroAmount);
        }

        amounts.push(atom.amount);
        let blinding = Scalar::random(&mut rand::thread_rng());
        blindings.push(blinding);

        let balance = asset_balances.entry(atom.asset_id).or_insert(0i128);
        *balance += if is_input {
            atom.amount as i128
        } else {
            -(atom.amount as i128)
        };

        let h_a = hash_to_curve(&atom.asset_id);
        let blinded_asset_id = (h_a + pc_gens.B_blinding * blinding).compress();
        let commitment =
            (pc_gens.B * Scalar::from(atom.amount) + pc_gens.B_blinding * blinding + h_a)
                .compress();

        asset_commitments
            .entry(blinded_asset_id)
            .and_modify(|e| {
                *e = (e.decompress().unwrap()
                    + commitment.decompress().unwrap()
                        * if is_input { Scalar::ONE } else { -Scalar::ONE })
                .compress()
            })
            .or_insert(commitment);
    }

    // Check balance for each asset
    for (_, balance) in asset_balances {
        if balance != 0 {
            return Err(Error::BalanceCheckFailed);
        }
    }

    let mut transcript = create_transcript(COMMITMENT_TRANSCRIPT_LABEL);

    // Append all asset commitments to the transcript
    for (blinded_asset_id, commitment) in &asset_commitments {
        transcript.append_message(b"blinded_asset_id", blinded_asset_id.as_bytes());
        transcript.append_message(b"commitment", commitment.as_bytes());
    }

    // Create a single aggregated range proof for all amounts
    let (bulletproof, _) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &amounts.iter().map(|&a| a as u64).collect::<Vec<_>>(),
        &blindings,
        64,
    )
    .map_err(|e| {
        Error::RangeProofError(format!("Failed to create aggregated range proof: {:?}", e))
    })?;

    Ok(TransactionCommitment {
        bulletproof,
        asset_commitments,
    })
}

pub fn verify(swap: &Swap) -> Result<(), Error> {
    if swap.inputs.len() > MAX_INPUTS {
        return Err(Error::TooManyInputs(swap.inputs.len()));
    }

    if swap.outputs.len() > MAX_OUTPUTS {
        return Err(Error::TooManyOutputs(swap.outputs.len()));
    }

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, MAX_INPUTS + MAX_OUTPUTS);

    let mut verifier_transcript = create_transcript(COMMITMENT_VERIFIER_LABEL);

    // Append all asset commitments to the transcript
    for (blinded_asset_id, commitment) in &swap.commitment.asset_commitments {
        verifier_transcript.append_message(b"blinded_asset_id", blinded_asset_id.as_bytes());
        verifier_transcript.append_message(b"commitment", commitment.as_bytes());
    }

    // Verify the bulletproof
    swap.commitment
        .bulletproof
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &swap
                .commitment
                .asset_commitments
                .values()
                .cloned()
                .collect::<Vec<_>>(),
            64,
        )
        .map_err(|_| Error::InvalidCommitment)?;

    // Verify that all asset commitments are non-zero
    for commitment in swap.commitment.asset_commitments.values() {
        if commitment == &CompressedRistretto::identity() {
            return Err(Error::ZeroAmount);
        }
    }

    Ok(())
}
