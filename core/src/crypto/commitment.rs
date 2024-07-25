use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use merlin::Transcript;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::crypto::*;
use crate::Hash;

#[derive(Debug, Clone)]
pub struct TransactionCommitment {
    pub bulletproof_low: RangeProof,
    pub bulletproof_high: RangeProof,
    pub commitments: Vec<CompressedRistretto>,
    pub asset_ids: Vec<Hash>,
}

fn create_transcript(label: &'static [u8]) -> Transcript {
    let mut transcript = Transcript::new(label);
    transcript.append_message(b"protocol-name", COMMITMENT_TRANSCRIPT_LABEL);
    transcript
}

pub fn commit(
    asset_ids: &[Hash],
    amounts: &[u128],
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
    let mut padded_amounts_low = Vec::with_capacity(party_capacity);
    let mut padded_amounts_high = Vec::with_capacity(party_capacity);
    let mut padded_blindings = blindings.to_vec();

    for &amount in amounts {
        padded_amounts_low.push(u64::try_from(amount & 0xFFFFFFFFFFFFFFFF).unwrap());
        padded_amounts_high.push(u64::try_from((amount >> 64) & 0xFFFFFFFFFFFFFFFF).unwrap());
    }

    while padded_amounts_low.len() < party_capacity {
        padded_amounts_low.push(0);
        padded_amounts_high.push(0);
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

    // Create the aggregated range proofs for low and high bits
    let (bulletproof_low, _) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript.clone(),
        &padded_amounts_low,
        &padded_blindings,
        64,
    )
    .map_err(|e| {
        format!(
            "Failed to create aggregated range proof for low bits: {:?}",
            e
        )
    })?;

    let (bulletproof_high, _) = RangeProof::prove_multiple(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &padded_amounts_high,
        &padded_blindings,
        64,
    )
    .map_err(|e| {
        format!(
            "Failed to create aggregated range proof for high bits: {:?}",
            e
        )
    })?;

    Ok(TransactionCommitment {
        bulletproof_low,
        bulletproof_high,
        commitments,
        asset_ids: asset_ids.to_vec(),
    })
}

pub fn verify(commitment: &TransactionCommitment) -> Result<(), &'static str> {
    if commitment.commitments.len() != commitment.asset_ids.len() {
        return Err("Mismatched commitment and asset_id lengths");
    }

    let pc_gens = PedersenGens::default();
    let party_capacity = commitment.commitments.len().next_power_of_two();
    let bp_gens = BulletproofGens::new(64, party_capacity);

    // Verify the aggregated range proofs
    let mut verifier_transcript = create_transcript(COMMITMENT_VERIFIER_LABEL);
    for (commitment, asset_id) in commitment
        .commitments
        .iter()
        .zip(commitment.asset_ids.iter())
    {
        verifier_transcript.append_message(b"asset_id", asset_id);
        verifier_transcript.append_message(b"commitment", commitment.as_bytes());
    }

    commitment
        .bulletproof_low
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript.clone(),
            &commitment.commitments,
            64,
        )
        .map_err(|_| "Aggregated range proof verification failed for low bits")?;

    commitment
        .bulletproof_high
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &commitment.commitments,
            64,
        )
        .map_err(|_| "Aggregated range proof verification failed for high bits")?;

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
    fn test_create_commitment(
        #[strategy(1..=16usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u128::MAX, #_num_assets))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment
        let commitment = commit(&asset_ids, &amounts, &blindings)?;

        prop_assert_eq!(
            commitment.commitments.len(),
            commitment.asset_ids.len(),
            "Mismatching number of assets and commitments"
        );
    }

    #[proptest(cases = 1)]
    fn test_create_commitment_failure_mismatched_assets(
        #[strategy(2..=16usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u128::MAX, #_num_assets))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets + 1))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment with mismatched asset_ids and blindings
        let result = commit(&asset_ids, &amounts, &blindings);

        prop_assert!(
            result.is_err(),
            "Expected an error due to mismatched asset_ids and blindings lengths"
        );
    }

    #[proptest(cases = 1)]
    fn test_create_commitment_failure_mismatched_amounts(
        #[strategy(2..=16usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u128::MAX, #_num_assets + 1))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment with mismatched amounts
        let result = commit(&asset_ids, &amounts, &blindings);

        prop_assert!(
            result.is_err(),
            "Expected an error due to mismatched amounts length"
        );
    }

    #[proptest(cases = 1)]
    fn test_check_balance_failure_mismatched_amounts(
        #[strategy(2..=8usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(1..u64::MAX as u128, #_num_assets))] input_amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] input_blindings: Vec<Scalar>,
        #[strategy(vec(scalar(), #_num_assets))] output_blindings: Vec<Scalar>,
    ) {
        // Create input commitment
        let input_commitment = commit(&asset_ids, &input_amounts, &input_blindings).unwrap();

        // Create output commitment with slightly different amounts
        let mut output_amounts = input_amounts.clone();
        output_amounts[0] += 1; // Ensure at least one amount is different

        let output_commitment = commit(&asset_ids, &output_amounts, &output_blindings).unwrap();

        // Check balance
        let result = check_balance(&[input_commitment], &[output_commitment]);

        prop_assert!(
            result.is_err(),
            "Expected an error due to mismatched amounts between inputs and outputs"
        );
    }

    #[proptest(cases = 1)]
    fn test_u128_amount_processing(
        #[strategy(2..=10usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec((u64::MAX as u128)..u128::MAX, #_num_assets))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] blindings: Vec<Scalar>,
    ) {
        // Create commitment
        let commitment_result = commit(&asset_ids, &amounts, &blindings);
        prop_assert!(commitment_result.is_ok(), "Failed to create commitment");
        let commitment = commitment_result.unwrap();

        // Verify commitment
        let verify_result = verify(&commitment);
        prop_assert!(verify_result.is_ok(), "Failed to verify commitment");

        // Check that we have the correct number of commitments
        prop_assert_eq!(
            commitment.commitments.len(),
            3,
            "Incorrect number of commitments"
        );

        // Attempt to create an invalid commitment (more amounts than blindings)
        let invalid_amounts = vec![0u128, 1u128, 2u128, 3u128];
        let invalid_result = commit(&asset_ids, &invalid_amounts, &blindings);
        prop_assert!(
            invalid_result.is_err(),
            "Expected an error for mismatched input lengths"
        );

        // Test balance check
        let inputs = vec![commitment.clone()];
        let outputs = vec![commitment];
        let balance_result = check_balance(&inputs, &outputs);
        prop_assert!(balance_result.is_ok(), "Balance check failed");
    }
}
