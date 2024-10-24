use std::collections::HashMap;

use curve25519_dalek::Scalar;
use proptest::prelude::*;
use rand::rngs::OsRng;

use crate::{protocol::*, testing::distribute, unwind_panic};

#[derive(Debug)]
pub struct Append<const I: usize, const O: usize> {
    pub inputs: [SealedNote; I],
    pub outputs: [Note; O],
}

impl<const I: usize, const O: usize> Append<I, O> {
    pub fn is_valid(&self) -> bool {
        let mut pre = HashMap::new();
        let mut post = HashMap::new();

        for input in self.inputs.iter() {
            *pre.entry((input.note.asset_id, input.note.asset_name))
                .or_insert(0u128) += input.note.amount as u128;
        }

        for output in self.outputs.iter() {
            *post
                .entry((output.asset_id, output.asset_name))
                .or_insert(0u128) += output.amount as u128;
        }

        pre.values().sum::<u128>() == post.values().sum::<u128>()
    }
}

impl<const I: usize, const O: usize> EncodeFields for Append<I, O> {
    fn as_fields(&self) -> Vec<F> {
        let mut fields = Vec::new();

        for input in &self.inputs {
            fields.extend(input.note.as_fields());
        }

        for output in &self.outputs {
            fields.extend(output.as_fields());
        }

        fields
    }
}

impl<const I: usize, const O: usize> Arbitrary for Append<I, O> {
    type Parameters = SecretKey;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(secret_key: Self::Parameters) -> Self::Strategy {
        // Use `distribute` to generate balanced inputs and outputs
        distribute(I, O)
            .prop_map(move |(input_notes, output_notes)| {
                // Sign each input note to create a `SealedNote`
                let inputs = input_notes
                    .into_iter()
                    .map(|note| {
                        let r = Scalar::random(&mut OsRng);
                        let blinded = secret_key.public().blind(note.clone(), &r).unwrap();
                        let blind_sig = secret_key.sign_blinded(blinded);
                        let signature = secret_key.public().unblind(blind_sig, r);

                        SealedNote {
                            issuing_key: secret_key.public(),
                            host: "localhost".to_string(),
                            port: 4000,
                            note,
                            signature,
                        }
                    })
                    .collect::<Vec<_>>();

                // Ensure inputs and outputs have correct sizes
                Append {
                    inputs: inputs.try_into().unwrap(),
                    outputs: output_notes.try_into().unwrap(),
                }
            })
            .boxed()
    }

    fn arbitrary() -> Self::Strategy {
        any::<SecretKey>()
            .prop_flat_map(Self::arbitrary_with)
            .boxed()
    }
}

#[derive(Debug)]
pub struct Atom {
    pub commitment: HashOutTarget,
    pub fields: Vec<Target>,
}

impl Atom {
    pub fn amount(&self) -> Target {
        self.fields[0]
    }

    pub fn asset_id(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.fields[1..5].try_into().unwrap(),
        }
    }

    pub fn asset_name(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.fields[5..9].try_into().unwrap(),
        }
    }
}

pub struct Circuit<const I: usize, const O: usize> {
    data: CircuitData,
    inputs: [Atom; I],
    outputs: [Atom; O],
}

impl<const I: usize, const O: usize> Sealable for Append<I, O> {
    type Circuit = Circuit<I, O>;

    fn circuit() -> Self::Circuit {
        let mut builder = circuit_builder();

        let mut inputs = vec![];
        let mut outputs = vec![];

        // Create input and output atoms using the refined circuit_seal_note
        for _ in 0..I {
            let (commitment, fields) = circuit_seal_note(&mut builder);

            inputs.push(Atom {
                commitment,
                fields: fields.try_into().unwrap(),
            });
        }

        for _ in 0..O {
            let (commitment, fields) = circuit_seal_note(&mut builder);
            outputs.push(Atom {
                commitment,
                fields: fields.try_into().unwrap(),
            });
        }

        // For each unique asset_id/asset_name pair, verify sum of amounts matches
        for i in 0..I {
            let mut input_sum = (&inputs[i]).amount();

            for j in (i + 1)..I {
                let other = &inputs[j];

                let assets_match = assets_match(
                    &mut builder,
                    inputs[i].asset_id(),
                    inputs[i].asset_name(),
                    other.asset_id(),
                    other.asset_name(),
                );

                // Add amount if assets match
                let should_add = builder.mul(assets_match.target, other.amount());
                input_sum = builder.add(input_sum, should_add);
            }

            // Sum up all output amounts for this asset
            let mut output_sum = builder.zero();

            for output in &outputs {
                let assets_match = assets_match(
                    &mut builder,
                    inputs[i].asset_id(),
                    inputs[i].asset_name(),
                    output.asset_id(),
                    output.asset_name(),
                );

                // Add amount if assets match
                let should_add = builder.mul(assets_match.target, output.amount());
                output_sum = builder.add(output_sum, should_add);
            }

            // Verify sums match
            builder.connect(input_sum, output_sum);
        }

        let circuit = Circuit {
            data: builder.build::<C>(),
            inputs: inputs.try_into().unwrap(),
            outputs: outputs.try_into().unwrap(),
        };

        circuit
    }

    fn circuit_data() -> CircuitData {
        Self::circuit().data
    }

    fn prove(&self) -> Result<Proof, Error> {
        let circuit = Self::circuit();
        let mut pw = PartialWitness::new();

        // Set input values
        for (i, input) in self.inputs.iter().enumerate() {
            let commitment = PoseidonHash::hash_no_pad(&input.note.as_fields_with_prefix());

            pw.set_hash_target(circuit.inputs[i].commitment, commitment);
            pw.set_target_arr(&circuit.inputs[i].fields, &input.note.as_fields());
        }

        // Set output values
        for (i, output) in self.outputs.iter().enumerate() {
            let commitment = PoseidonHash::hash_no_pad(&output.as_fields_with_prefix());

            pw.set_hash_target(circuit.outputs[i].commitment, commitment);
            pw.set_target_arr(&circuit.outputs[i].fields, &output.as_fields());
        }

        unwind_panic!(circuit.data.prove(pw).map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        }))
    }
}

fn assets_match(
    builder: &mut CircuitBuilder,
    id1: HashOutTarget,
    name1: HashOutTarget,
    id2: HashOutTarget,
    name2: HashOutTarget,
) -> BoolTarget {
    let mut assets_match = builder._true();

    for k in 0..4 {
        let id_match = builder.is_equal(id1.elements[k], id2.elements[k]);
        let name_match = builder.is_equal(name1.elements[k], name2.elements[k]);
        let both_match = builder.and(id_match, name_match);
        assets_match = builder.and(assets_match, both_match);
    }

    assets_match
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest(cases = 50)]
    fn test_arbitrary_is_always_valid(append: Append<4, 4>) {
        prop_assert!(append.is_valid());
    }
}
