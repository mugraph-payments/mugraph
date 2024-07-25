use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use merlin::Transcript;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::crypto::*;
use crate::error::Error;
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
) -> Result<TransactionCommitment, Error> {
    if asset_ids.len() != amounts.len() || amounts.len() != blindings.len() {
        return Err(Error::MismatchedInputLengths);
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
        padded_amounts_low
            .push(u64::try_from(amount & 0xFFFFFFFFFFFFFFFF).map_err(|_| Error::InvalidAmount)?);
        padded_amounts_high.push(
            u64::try_from((amount >> 64) & 0xFFFFFFFFFFFFFFFF).map_err(|_| Error::InvalidAmount)?,
        );
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
        Error::RangeProofError(format!(
            "Failed to create aggregated range proof for low bits: {:?}",
            e
        ))
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
        Error::RangeProofError(format!(
            "Failed to create aggregated range proof for high bits: {:?}",
            e
        ))
    })?;

    Ok(TransactionCommitment {
        bulletproof_low,
        bulletproof_high,
        commitments,
        asset_ids: asset_ids.to_vec(),
    })
}

pub fn verify(commitment: &TransactionCommitment) -> Result<(), Error> {
    if commitment.commitments.len() != commitment.asset_ids.len() {
        return Err(Error::InvalidTransactionCommitment);
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
        .map_err(|_| Error::BulletproofVerificationFailed)?;

    commitment
        .bulletproof_high
        .verify_multiple(
            &bp_gens,
            &pc_gens,
            &mut verifier_transcript,
            &commitment.commitments,
            64,
        )
        .map_err(|_| Error::BulletproofVerificationFailed)?;

    // Verify that each commitment is well-formed
    for (commitment, asset_id) in commitment
        .commitments
        .iter()
        .zip(commitment.asset_ids.iter())
    {
        let h_a = hash_to_curve(asset_id);

        // Decompress the commitment point
        let commitment_point = commitment
            .decompress()
            .ok_or(Error::InvalidPointCompression)?;

        // Check that the commitment point is not the identity (all zeros)
        if commitment_point == RistrettoPoint::identity() {
            return Err(Error::InvalidTransactionCommitment);
        }

        // Check that the commitment point is not equal to h_a
        // This ensures that the commitment includes some amount and/or blinding factor
        if commitment_point == h_a {
            return Err(Error::InvalidTransactionCommitment);
        }

        // Note: We can't fully verify C = aG + bH + h_a without knowing a and b
    }

    Ok(())
}

pub fn check_balance(
    inputs: &[TransactionCommitment],
    outputs: &[TransactionCommitment],
) -> Result<(), Error> {
    let mut balance = HashMap::new();

    // Sum up input commitments
    for input in inputs {
        for (commitment, asset_id) in input.commitments.iter().zip(input.asset_ids.iter()) {
            let entry = balance
                .entry(asset_id)
                .or_insert(CompressedRistretto::identity());
            *entry = (entry.decompress().ok_or(Error::InvalidPointCompression)?
                + commitment
                    .decompress()
                    .ok_or(Error::InvalidPointCompression)?)
            .compress();
        }
    }

    // Subtract output commitments
    for output in outputs {
        for (commitment, asset_id) in output.commitments.iter().zip(output.asset_ids.iter()) {
            let entry = balance
                .entry(asset_id)
                .or_insert(CompressedRistretto::identity());
            *entry = (entry.decompress().ok_or(Error::InvalidPointCompression)?
                - commitment
                    .decompress()
                    .ok_or(Error::InvalidPointCompression)?)
            .compress();
        }
    }

    // Check if all balances are zero
    for (_, commitment) in balance {
        if commitment != CompressedRistretto::identity() {
            return Err(Error::BalanceCheckFailed);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use curve25519_dalek::scalar::Scalar;
    use proptest::{collection::vec, prelude::*};
    use test_strategy::proptest;

    #[proptest(cases = 1)]
    fn test_create_commitment(
        #[strategy(2..=8usize)] _num_assets: usize,
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
        #[strategy(2..=8usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u128::MAX, #_num_assets))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets + 1))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment with mismatched asset_ids and blindings
        let result = commit(&asset_ids, &amounts, &blindings);

        prop_assert!(
            matches!(result, Err(Error::MismatchedInputLengths)),
            "Expected a MismatchedInputLengths error due to mismatched asset_ids and blindings lengths"
        );
    }

    #[proptest(cases = 1)]
    fn test_create_commitment_failure_mismatched_amounts(
        #[strategy(2..=8usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec(0..u128::MAX, #_num_assets + 1))] amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] blindings: Vec<Scalar>,
    ) {
        // Test creating an aggregated asset commitment with mismatched amounts
        let result = commit(&asset_ids, &amounts, &blindings);

        prop_assert!(
            matches!(result, Err(Error::MismatchedInputLengths)),
            "Expected a MismatchedInputLengths error due to mismatched amounts length"
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
        let input_commitment = commit(&asset_ids, &input_amounts, &input_blindings)?;

        // Create output commitment with slightly different amounts
        let mut output_amounts = input_amounts.clone();
        output_amounts[0] += 1; // Ensure at least one amount is different

        let output_commitment = commit(&asset_ids, &output_amounts, &output_blindings)?;

        // Check balance
        let result = check_balance(&[input_commitment], &[output_commitment]);

        prop_assert!(
            matches!(result, Err(Error::BalanceCheckFailed)),
            "Expected a BalanceCheckFailed error due to mismatched amounts between inputs and outputs"
        );
    }

    #[proptest(cases = 1)]
    fn test_u128_amount_processing_failure(
        #[strategy(2..=8usize)] _num_assets: usize,
        #[strategy(vec(any::<Hash>(), #_num_assets))] asset_ids: Vec<Hash>,
        #[strategy(vec((u64::MAX as u128)..u128::MAX, #_num_assets))] input_amounts: Vec<u128>,
        #[strategy(vec(scalar(), #_num_assets))] input_blindings: Vec<Scalar>,
        #[strategy(vec(scalar(), #_num_assets))] output_blindings: Vec<Scalar>,
    ) {
        // Create input commitment
        let input_commitment = commit(&asset_ids, &input_amounts, &input_blindings)?;

        // Create output commitment with slightly different amounts
        let mut output_amounts = input_amounts.clone();
        output_amounts[0] += 1; // Ensure at least one amount is different

        let output_commitment = commit(&asset_ids, &output_amounts, &output_blindings)?;

        // Check balance
        let result = check_balance(&[input_commitment], &[output_commitment]);

        prop_assert!(
            matches!(result, Err(Error::BalanceCheckFailed)),
            "Expected a BalanceCheckFailed error due to mismatched amounts between inputs and outputs"
        );
    }
}
