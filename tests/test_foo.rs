use curve25519_dalek::scalar::Scalar;
use mugraph::{protocol::*, unwind_panic, Error};
use proptest::prelude::*;
use test_strategy::proptest;

/// Native blind function using curve25519-dalek.
///
/// This is the function that we want to replicate in our circuit.
pub fn blind_native(input: Hash, r: Hash) -> Hash {
    let r: Scalar = r.into();
    (hash_to_curve(input).unwrap() + r * G).into()
}

/// This is the zero-knowledge version of the `blind_native` function.
///
/// This should work exactly equal to the `blind_native` version, while keeping into consideration
/// the different fields (Goldilocks field for the ZK circuit).
fn blind_zk(input: Hash, r: Hash) -> Result<Proof, Error> {
    let mut builder = circuit_builder();

    // Add input point as virtual target
    let data = builder.add_virtual_hash();

    // Add blinding factor as virtual target
    let r_target = builder.add_virtual_hash();

    // Compute hash_to_curve(input) + r*G
    let input_point = circuit_hash_to_curve(&mut builder, &data.elements);
    let r_point = builder.ec_mul(&r_target.elements, &G);
    let result = builder.ec_add(&input_point.elements, &r_point.elements);

    // Add result as public input
    let public_result = builder.add_virtual_hash_public_input();
    builder.connect_hashes(&result, public_result);

    let mut pw = PartialWitness::new();
    pw.set_hash_target(data, input.into());
    pw.set_hash_target(r_target, r.into());

    let circuit = builder.build::<C>();

    let proof = unwind_panic!(circuit.prove(pw)).map_err(|e| Error::CryptoError {
        kind: "Proof generation failed".to_string(),
        reason: e.to_string(),
    })?;

    Ok(Proof {
        proof: proof.proof,
        public_inputs: proof.public_inputs,
    })
}

#[proptest]
fn test_foo_blind_foo(input: Hash, r: Hash) {
    prop_assert_eq!(
        Hash::from_fields(&blind_zk(input, r)?.public_inputs)?,
        blind_native(input, r)
    );
}
