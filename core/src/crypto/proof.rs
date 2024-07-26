use std::collections::HashMap;

use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        circuit_data::VerifierCircuitData, config::PoseidonGoldilocksConfig,
    },
};

use crate::{error::Error, types::*};

pub fn prove_swap(inputs: &[Note], outputs: &[UnblindedNote]) -> Result<Proof, Error> {
    if inputs.len() > 8 {
        return Err(Error::TooManyInputs(inputs.len()));
    }

    if outputs.len() > 8 {
        return Err(Error::TooManyOutputs(outputs.len()));
    }

    let mut builder =
        CircuitBuilder::<GoldilocksField, 2>::new(CircuitConfig::standard_recursion_config());
    let zero = builder.zero();

    let mut input_sums = HashMap::new();
    let mut output_sums = HashMap::new();

    let amount_lut = builder.add_lookup_table_from_fn(|n| n, &(0..=u16::MAX).collect::<Vec<u16>>());

    let mut pw = PartialWitness::new();

    for input in inputs {
        let amount_target = builder.add_virtual_target();
        pw.set_target(
            amount_target,
            GoldilocksField::from_canonical_u64(input.amount),
        );

        let output_target = builder.add_lookup_from_index(amount_target, amount_lut);
        builder.connect(amount_target, output_target);

        let sum = input_sums.entry(input.asset_id).or_insert(builder.zero());
        *sum = builder.add(*sum, output_target);
    }

    for output in outputs {
        let amount_target = builder.add_virtual_target();
        pw.set_target(
            amount_target,
            GoldilocksField::from_canonical_u64(output.amount),
        );

        let output_target = builder.add_lookup_from_index(amount_target, amount_lut);
        builder.connect(amount_target, output_target);

        let is_zero = builder.is_equal(output_target, zero);
        let is_not_zero = builder.not(is_zero);
        builder.assert_one(is_not_zero.target);

        let sum = output_sums.entry(output.asset_id).or_insert(builder.zero());
        *sum = builder.add(*sum, output_target);
    }

    if input_sums.len() > 8 || output_sums.len() > 8 {
        return Err(Error::TooManyAssets(std::cmp::max(
            input_sums.len(),
            output_sums.len(),
        )));
    }

    for (asset_id, input_sum) in input_sums.iter() {
        let output_sum = output_sums.get(asset_id).unwrap_or(&zero);
        let is_equal = builder.is_equal(*input_sum, *output_sum);
        builder.assert_one(is_equal.target);
    }

    // Check that all input asset IDs are in the output sums
    for asset_id in input_sums.keys() {
        let contains_key = builder.add_virtual_bool_target_safe();
        pw.set_bool_target(contains_key, output_sums.contains_key(asset_id));
        builder.assert_one(contains_key.target);
    }

    let circuit_data = builder.build::<PoseidonGoldilocksConfig>();

    let proof = circuit_data.prove(pw)?;

    Ok(Proof {
        proof,
        data: VerifierCircuitData {
            verifier_only: circuit_data.verifier_only,
            common: circuit_data.common,
        },
    })
}

/// Verifies a Zero-Knowledge Proof for a Swap.
pub fn verify_swap_proof(swap: Swap) -> Result<(), Error> {
    swap.proof.data.verify(swap.proof.proof)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_keypair, hash_to_curve, schnorr};

    #[test]
    fn test_prove_swap() {
        // Generate a simulated key pair
        let (secret_key, _) = generate_keypair();

        let inputs = vec![
            Note {
                asset_id: Hash::default(),
                amount: 100,
                signature: schnorr::sign(&secret_key, &[0u8; 32]), // Sign with a dummy message
            },
            Note {
                asset_id: Hash::default(),
                amount: 50,
                signature: schnorr::sign(&secret_key, &[1u8; 32]), // Sign with a different dummy message
            },
        ];

        let outputs = vec![UnblindedNote {
            asset_id: Hash::default(),
            amount: 150,
            nonce: Hash::default(),
        }];

        let result = prove_swap(&inputs, &outputs);
        assert!(result.is_ok(), "prove_swap failed: {:?}", result.err());
        let proof = result.unwrap();
        let swap = Swap {
            proof,
            inputs: inputs.into_iter().map(|n| n.signature).collect(),
            outputs: outputs
                .into_iter()
                .map(|n| {
                    hash_to_curve(
                        &[&n.asset_id, n.amount.to_le_bytes().as_ref(), &n.nonce].concat(),
                    )
                })
                .collect(),
        };

        let verification_result = verify_swap_proof(swap);
        assert!(
            verification_result.is_ok(),
            "Proof verification failed: {:?}",
            verification_result.err()
        );
    }
}
