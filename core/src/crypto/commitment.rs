use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use merlin::Transcript;
use std::collections::HashMap;

use crate::crypto::*;
use crate::Hash;

#[derive(Debug, Clone)]
pub struct TransactionCommitment {
    pub bulletproof: RangeProof,
    pub commitments: Vec<CompressedRistretto>,
    pub asset_ids: Vec<Hash>,
}

fn create_transcript(label: &'static [u8]) -> Transcript {
    let mut transcript = Transcript::new(label);
    transcript.append_message(b"protocol-name", b"AggregatedAssetCommitment");
    transcript
}

pub fn create_aggregated_asset_commitment(
    asset_ids: &[Hash],
    amounts: &[u64],
    blindings: &[Scalar],
) -> Result<TransactionCommitment, String> {
    if asset_ids.len() != amounts.len() || amounts.len() != blindings.len() {
        return Err(format!(
            "Mismatched input lengths: asset_ids={}, amounts={}, blindings={}",
            asset_ids.len(),
            amounts.len(),
            blindings.len()
        ));
    }

    let pc_gens = PedersenGens::default();

    // Round up to the next power of 2
    let party_capacity = amounts.len().next_power_of_two();
    let bp_gens = BulletproofGens::new(64, party_capacity);

    // Pad amounts and blindings to the next power of 2
    let mut padded_amounts = amounts.to_vec();
    let mut padded_blindings = blindings.to_vec();
    while padded_amounts.len() < party_capacity {
        padded_amounts.push(0);
        padded_blindings.push(Scalar::ZERO);
    }

    // Create individual commitments
    let mut commitments = Vec::with_capacity(amounts.len());
    let mut transcript = create_transcript(COMMITMENT_TRANSCRIPT_LABEL);

    for i in 0..amounts.len() {
        let h_a = hash_to_curve(&asset_ids[i]);
        let commitment =
            (pc_gens.B * Scalar::from(amounts[i]) + pc_gens.B_blinding * blindings[i] + h_a)
                .compress();
        commitments.push(commitment);

        transcript.append_message(b"asset_id", &asset_ids[i]);
        transcript.append_message(b"commitment", commitment.as_bytes());
    }

    // Create the aggregated range proof
    let (bulletproof, _) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &padded_amounts,
        &padded_blindings,
        64,
    )
    .map_err(|e| format!("Failed to create aggregated range proof: {:?}", e))?;

    Ok(TransactionCommitment {
        bulletproof,
        commitments,
        asset_ids: asset_ids.to_vec(),
    })
}

pub fn verify_aggregated_asset_commitment(
    commitment: &TransactionCommitment,
) -> Result<(), &'static str> {
    if commitment.commitments.len() != commitment.asset_ids.len() {
        return Err("Mismatched commitment and asset_id lengths");
    }

    let pc_gens = PedersenGens::default();
    let party_capacity = commitment.commitments.len().next_power_of_two();
    let bp_gens = BulletproofGens::new(64, party_capacity);

    // Verify the aggregated range proof
    let mut verifier_transcript = create_transcript(b"AggregatedAssetCommitment");
    for (commitment, asset_id) in commitment
        .commitments
        .iter()
        .zip(commitment.asset_ids.iter())
    {
        verifier_transcript.append_message(b"asset_id", asset_id);
        verifier_transcript.append_message(b"commitment", commitment.as_bytes());
    }
    commitment
        .bulletproof
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &commitment.commitments,
            64,
        )
        .map_err(|_| "Aggregated range proof verification failed")?;

    // Verify that each commitment is well-formed
    for (commitment, asset_id) in commitment
        .commitments
        .iter()
        .zip(commitment.asset_ids.iter())
    {
        let h_a = hash_to_curve(asset_id);

        // Decompress the commitment point
        let commitment_point = commitment.decompress().ok_or("Invalid commitment point")?;

        // Check that the commitment point is not the identity (all zeros)
        if commitment_point == RistrettoPoint::identity() {
            return Err("Commitment cannot be the identity point");
        }

        // Check that the commitment point is not equal to h_a
        // This ensures that the commitment includes some amount and/or blinding factor
        if commitment_point == h_a {
            return Err("Commitment cannot be equal to the asset-specific generator");
        }

        // Note: We can't fully verify C = aG + bH + h_a without knowing a and b
    }

    Ok(())
}

pub fn check_balance(
    inputs: &[TransactionCommitment],
    outputs: &[TransactionCommitment],
) -> Result<(), &'static str> {
    let mut balance = HashMap::new();

    // Sum up input commitments
    for input in inputs {
        for (commitment, asset_id) in input.commitments.iter().zip(input.asset_ids.iter()) {
            let entry = balance
                .entry(asset_id)
                .or_insert(CompressedRistretto::identity());
            *entry = (entry.decompress().ok_or("Invalid commitment point")?
                + commitment.decompress().ok_or("Invalid commitment point")?)
            .compress();
        }
    }

    // Subtract output commitments
    for output in outputs {
        for (commitment, asset_id) in output.commitments.iter().zip(output.asset_ids.iter()) {
            let entry = balance
                .entry(asset_id)
                .or_insert(CompressedRistretto::identity());
            *entry = (entry.decompress().ok_or("Invalid commitment point")?
                - commitment.decompress().ok_or("Invalid commitment point")?)
            .compress();
        }
    }

    // Check if all balances are zero
    for (_, commitment) in balance {
        if commitment != CompressedRistretto::identity() {
            return Err("Balance check failed");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::testing::*;
    use ::proptest::{collection::vec, prelude::*};
    use curve25519_dalek::scalar::Scalar;
    use test_strategy::proptest;

    #[proptest(cases = 1)]
    fn test_create_aggregated_asset_commitment_success(
        #[strategy(1..=10usize)] num_assets: usize,
        #[strategy(vec(any::<Hash>(), #num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u64::MAX, #num_assets))] amounts: Vec<u64>,
        #[strategy(vec(scalar(), #num_assets))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment
        let result = create_aggregated_asset_commitment(&asset_ids, &amounts, &blindings);

        prop_assert!(
            result.is_ok(),
            "Failed to create aggregated asset commitment"
        );

        let commitment = result.unwrap();
        prop_assert_eq!(
            commitment.commitments.len(),
            num_assets,
            "Incorrect number of commitments"
        );
        prop_assert_eq!(
            commitment.asset_ids.len(),
            num_assets,
            "Incorrect number of asset IDs"
        );
    }

    #[proptest(cases = 1)]
    fn test_create_aggregated_asset_commitment_failure(
        #[strategy(2..=10usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u64::MAX, #_num_assets + 1))] amounts: Vec<u64>,
        #[strategy(vec(scalar(), #_num_assets))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment with invalid data
        let result = create_aggregated_asset_commitment(&asset_ids, &amounts, &blindings);

        prop_assert!(
            result.is_err(),
            "Expected an error due to mismatched input lengths"
        );
        prop_assert!(
            result.unwrap_err().contains("Mismatched input lengths"),
            "Unexpected error message"
        );
    }
}
