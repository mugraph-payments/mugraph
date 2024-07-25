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
    pub proof: RangeProof,
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
        proof: bulletproof,
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
        .proof
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::op::swap::*;
    use ::proptest::prelude::*;
    use proptest::{prelude::prop::collection::vec, sample::SizeRange};
    use test_strategy::proptest;

    fn atoms(
        input_count: impl Into<SizeRange>,
        output_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = (Vec<Atom>, Vec<Atom>)> {
        (
            vec(any::<Atom>(), input_count.into()),
            vec(any::<Atom>(), output_count.into()),
        )
    }

    #[proptest(cases = 1, fork = false)]
    fn test_balance_check(#[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>)) {
        let (inputs, outputs) = atoms;
        let result = commit(&inputs, &outputs);
        prop_assert!(result.is_ok());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_balance_check_failure(#[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>)) {
        let (mut inputs, outputs) = atoms;
        if !inputs.is_empty() {
            inputs[0].amount += 1; // Ensure imbalance
        }
        let result = commit(&inputs, &outputs);
        prop_assert!(result.is_err());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_range_proof_validity(#[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>)) {
        let (inputs, outputs) = atoms;
        let commitment = commit(&inputs, &outputs).unwrap();
        let swap = create_swap_from_commitment(commitment, inputs.len(), outputs.len());
        let result = verify(&swap);
        prop_assert!(result.is_ok());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_zero_amount_check(#[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>)) {
        let (mut inputs, outputs) = atoms;
        if !inputs.is_empty() {
            inputs[0].amount = 0;
        }
        let result = commit(&inputs, &outputs);
        prop_assert!(result.is_err());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_input_output_limits(
        #[strategy(atoms(16..=20, 16..=20))] atoms: (Vec<Atom>, Vec<Atom>),
    ) {
        let (inputs, outputs) = atoms;
        let result = commit(&inputs, &outputs);
        prop_assert!(result.is_err());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_commitment_uniqueness(#[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>)) {
        let (inputs, outputs) = atoms;
        let commitment1 = commit(&inputs, &outputs).unwrap();
        let commitment2 = commit(&inputs, &outputs).unwrap();

        prop_assert_ne!(commitment1.proof.to_bytes(), commitment2.proof.to_bytes());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_asset_id_blinding(
        #[strategy(any::<Atom>())] atom1: Atom,
        #[strategy(any::<Atom>())] mut atom2: Atom,
    ) {
        atom2.asset_id = atom1.asset_id;
        atom2.amount = atom1.amount;

        let commitment1 = commit(&[atom1.clone()], &[]).unwrap();
        let commitment2 = commit(&[atom2], &[]).unwrap();

        prop_assert_ne!(
            commitment1.asset_commitments.keys().next().unwrap(),
            commitment2.asset_commitments.keys().next().unwrap()
        );
    }

    #[proptest(cases = 1, fork = false)]
    fn test_valid_bulletproof_verification(
        #[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>),
    ) {
        let (inputs, outputs) = atoms;
        let commitment = commit(&inputs, &outputs).unwrap();
        let swap = create_swap_from_commitment(commitment, inputs.len(), outputs.len());

        // Verify valid commitment
        prop_assert!(verify(&swap).is_ok());
    }

    #[proptest(cases = 1, fork = false)]
    fn test_invalid_bulletproof_verification(
        #[strategy(atoms(1..=8, 1..=8))] atoms: (Vec<Atom>, Vec<Atom>),
        #[strategy(atoms(1..=8, 1..=8))] other_atoms: (Vec<Atom>, Vec<Atom>),
    ) {
        prop_assume!(atoms != other_atoms);

        let (inputs, outputs) = atoms;
        let (other_inputs, other_outputs) = other_atoms;
        let commitment = commit(&inputs, &outputs).unwrap();
        let other_commitment = commit(&other_inputs, &other_outputs).unwrap();
        let mut swap = create_swap_from_commitment(commitment, inputs.len(), outputs.len());

        // Modify bulletproof to make it invalid
        swap.commitment.proof = other_commitment.proof;

        // Ensure verification fails with invalid bulletproof
        prop_assert!(verify(&swap).is_err());
    }

    // Helper function to create a Swap from a TransactionCommitment
    fn create_swap_from_commitment(
        commitment: TransactionCommitment,
        input_count: usize,
        output_count: usize,
    ) -> Swap {
        let mut rng = rand::thread_rng();

        let inputs: Vec<Input> = (0..input_count)
            .map(|_| {
                let private_key = Scalar::random(&mut rng);
                let nullifier = RistrettoPoint::random(&mut rng);
                let blinded_point = RistrettoPoint::random(&mut rng);
                let (_signed_point, dleq_proof) = dh::sign_blinded(&private_key, &blinded_point);

                Input {
                    nullifier,
                    dleq_proof,
                }
            })
            .collect();

        let outputs: Vec<Output> = (0..output_count)
            .map(|_| Output {
                blinded_secret: RistrettoPoint::random(&mut rng),
            })
            .collect();

        let witnesses: Vec<Signature> = inputs
            .iter()
            .map(|input| {
                let private_key = Scalar::random(&mut rng);
                let nullifier = input.nullifier.clone().compress();
                let message = nullifier.as_bytes();

                schnorr::sign(&private_key, message)
            })
            .collect();

        Swap {
            inputs,
            outputs,
            commitment,
            witnesses,
        }
    }
}
