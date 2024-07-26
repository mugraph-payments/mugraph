use std::collections::HashMap;

use crate::{
    crypto::{hash_to_curve, proof::prove_swap},
    types::*,
    G,
};
use proptest::{collection::vec, prelude::*};

pub fn scalar() -> impl Strategy<Value = Scalar> {
    any::<[u8; 32]>().prop_map(Scalar::from_bytes_mod_order)
}

pub fn point() -> impl Strategy<Value = Point> {
    scalar().prop_map(|s| s * *G)
}

pub fn keypair() -> impl Strategy<Value = (SecretKey, PublicKey)> {
    scalar().prop_map(|s| (s, *G * s))
}

pub fn verified_swap() -> impl Strategy<Value = Swap> {
    let inputs = vec(any::<Note>(), 1..=8);
    let outputs = vec(any::<UnblindedNote>(), 1..=8);

    (inputs, outputs).prop_map(|(inputs, mut outputs)| {
        // Calculate total amounts per asset for inputs
        let mut input_totals = HashMap::new();

        for input in &inputs {
            *input_totals.entry(input.asset_id).or_insert(0) += input.amount;
        }

        // Create a list of asset_ids from input_totals
        let asset_ids: Vec<Hash> = input_totals.keys().cloned().collect();

        // Update outputs to match input totals
        for (i, output) in outputs.iter_mut().enumerate() {
            let asset_id = asset_ids[i % asset_ids.len()];
            let total = input_totals.get_mut(&asset_id).unwrap();

            if *total > 0 {
                output.asset_id = asset_id;
                output.amount = std::cmp::min(output.amount, *total);
                *total -= output.amount;
            } else {
                output.asset_id = asset_id;
                output.amount = 0;
            }
        }

        // Distribute any remaining amounts among existing outputs
        for (asset_id, remaining) in input_totals.iter() {
            if *remaining > 0 {
                for output in outputs.iter_mut() {
                    if output.asset_id == *asset_id && *remaining > 0 {
                        let additional = std::cmp::min(*remaining, u64::MAX - output.amount);
                        output.amount += additional;
                    }
                }
            }
        }

        let proof = prove_swap(&inputs, &outputs).expect("Failed to prove test transaction");

        let inputs = inputs.iter().map(|i| i.signature.clone()).collect();
        let outputs = outputs
            .into_iter()
            .map(|n| {
                hash_to_curve(&[&n.asset_id, n.amount.to_le_bytes().as_ref(), &n.nonce].concat())
            })
            .collect();

        Swap {
            inputs,
            outputs,
            proof,
        }
    })
}
