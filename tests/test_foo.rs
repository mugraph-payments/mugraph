use curve25519_dalek::scalar::Scalar;
use mugraph::{protocol::*, unwind_panic, Error};
use plonky2::{hash::hash_types::HashOutTarget, iop::target::Target};
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

    // Convert base point G to circuit form
    let g_circuit = {
        let g_coords = G.compress().to_bytes(); // Convert G to coordinates
        [
            builder.constant(F::from_canonical_u64(g_coords[0] as u64)),
            builder.constant(F::from_canonical_u64(g_coords[32] as u64)),
        ]
    };

    // Helper functions for EC operations
    fn point_double(builder: &mut CircuitBuilder, p: &[Target; 2]) -> [Target; 2] {
        let x = p[0];
        let y = p[1];

        // Lambda = (3x^2) / (2y)
        let x_squared = builder.mul(x, x);
        let three_x_squared = builder.mul_const(F::from_canonical_u64(3), x_squared);
        let two_y = builder.mul_const(F::from_canonical_u64(2), y);
        let lambda = builder.div(three_x_squared, two_y);

        // x_r = lambda^2 - 2x
        let lambda_squared = builder.mul(lambda, lambda);
        let two_x = builder.mul_const(F::from_canonical_u64(2), x);
        let x_r = builder.sub(lambda_squared, two_x);

        // y_r = lambda(x - x_r) - y
        let x_diff = builder.sub(x, x_r);
        let lambda_x_diff = builder.mul(lambda, x_diff);
        let y_r = builder.sub(lambda_x_diff, y);

        [x_r, y_r]
    }

    fn point_add(builder: &mut CircuitBuilder, p: &[Target; 2], q: &[Target; 2]) -> [Target; 2] {
        let x1 = p[0];
        let y1 = p[1];
        let x2 = q[0];
        let y2 = q[1];

        // Lambda = (y2 - y1) / (x2 - x1)
        let y_diff = builder.sub(y2, y1);
        let x_diff = builder.sub(x2, x1);
        let lambda = builder.div(y_diff, x_diff);

        // x_r = lambda^2 - x1 - x2
        let lambda_squared = builder.mul(lambda, lambda);
        let x_sum = builder.add(x1, x2);
        let x_r = builder.sub(lambda_squared, x_sum);

        // y_r = lambda(x1 - x_r) - y1
        let x_diff_result = builder.sub(x1, x_r);
        let lambda_x_diff = builder.mul(lambda, x_diff_result);
        let y_r = builder.sub(lambda_x_diff, y1);

        [x_r, y_r]
    }

    fn scalar_mul(builder: &mut CircuitBuilder, k: &[Target], p: &[Target; 2]) -> [Target; 2] {
        let zero = builder.zero();
        let mut result = [zero, zero];
        let temp = *p;

        for i in (0..k.len()).rev() {
            // Double
            result = point_double(builder, &result);

            // Add if current bit is 1
            let should_add = builder.add(k[i], zero);
            let added = point_add(builder, &result, &temp);
            let should_add = builder.is_equal(should_add, zero);

            result = [
                builder.select(should_add, added[0], result[0]),
                builder.select(should_add, added[1], result[1]),
            ];
        }

        result
    }

    // Compute hash_to_curve(input) + r*G
    let input_point = circuit_hash_to_curve(&mut builder, &data.elements);
    let r_point = scalar_mul(&mut builder, &r_target.elements[0..2], &g_circuit);
    let input_point_array = [input_point.elements[0], input_point.elements[1]];
    let result = point_add(&mut builder, &input_point_array, &r_point);

    // Convert result to HashOutTarget
    let result_hash = {
        let mut elements = [builder.zero(); 4];
        elements[0] = result[0];
        elements[1] = result[1];
        HashOutTarget { elements }
    };

    // Add result as public input
    let public_result = builder.add_virtual_hash_public_input();
    builder.connect_hashes(result_hash, public_result);

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
