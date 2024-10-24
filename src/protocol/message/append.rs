use std::collections::HashMap;

use curve25519_dalek::Scalar;
use proptest::{collection::vec, prelude::*, strategy::Just};
use rand::{prelude::*, rngs::OsRng};

use crate::{protocol::*, unwind_panic};

#[derive(Debug)]
pub struct Append<const I: usize, const O: usize> {
    pub inputs: [SealedNote; I],
    pub outputs: [Note; O],
}

impl<const I: usize, const O: usize> Append<I, O> {
    pub fn is_valid(&self) -> bool {
        let mut balance: HashMap<(Hash, Name), u128> = HashMap::new();

        for input in &self.inputs {
            let amount = input.note.amount;

            if amount == 0 {
                return false;
            }

            balance
                .entry((input.note.asset_id, input.note.asset_name))
                .and_modify(|x| *x += amount as u128)
                .or_insert(amount as u128);
        }

        for output in &self.outputs {
            let amount = output.amount;

            if amount == 0 {
                return false;
            }

            balance
                .entry((output.asset_id, output.asset_name))
                .and_modify(|x| *x -= amount as u128)
                .or_insert(u128::MAX);
        }

        // Check if all sums are zero
        balance.values().all(|&sum| sum == 0)
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
        let input_notes = (vec((any::<Hash>(), any::<SealedNote>()), I..=I)).prop_map(move |notes| {
            notes
                .into_iter()
                .map(|(r, note)| {
                    let r: Scalar = r.into();
                    let blinded = secret_key.public().blind(note.note.clone(), &r).unwrap();
                    let blind_sig = secret_key.sign_blinded(blinded);
                    let signature = secret_key.public().unblind(blind_sig, r);

                    SealedNote {
                        issuing_key: secret_key.public(),
                        host: note.host,
                        port: note.port,
                        note: note.note,
                        signature,
                    }
                })
                .collect::<Vec<_>>()
        });

        let outputs = input_notes.clone().prop_flat_map(move |inputs| {
            // Group inputs by asset_id and asset_name
            let mut balances: HashMap<(Hash, Name), u128> = HashMap::new();
            for input in &inputs {
                let key = (input.note.asset_id, input.note.asset_name);
                *balances.entry(key).or_default() += input.note.amount as u128;
            }

            // Generate O output notes that balance with inputs
            let mut outputs = Vec::with_capacity(O);
            let mut remaining_balances = balances.clone();

            // Distribute amounts across outputs
            while outputs.len() < O && !remaining_balances.is_empty() {
                let (&(asset_id, asset_name), balance) =
                    remaining_balances.iter_mut().next().unwrap();

                // Generate a random amount that doesn't exceed the remaining balance
                let amount = if *balance > 1 {
                    let max = (*balance).min(u64::MAX as u128) as u64;
                    if max > 1 {
                        (1..max).choose(&mut OsRng).unwrap()
                    } else {
                        1
                    }
                } else {
                    1
                };

                *balance -= amount as u128;

                // Remove if fully distributed
                if *balance == 0 {
                    remaining_balances.remove(&(asset_id, asset_name));
                }

                outputs.push(Note {
                    asset_id,
                    asset_name,
                    amount,
                    nonce: Hash::random(),
                });
            }

            // Fill remaining outputs with the last asset if needed
            while outputs.len() < O {
                let last = outputs.last().unwrap();

                outputs.push(Note {
                    asset_id: last.asset_id,
                    asset_name: last.asset_name,
                    amount: 1,
                    nonce: Hash::random(),
                });
            }

            Just(outputs)
        });

        // Combine inputs and outputs into Append
        (input_notes, outputs)
            .prop_map(|(inputs, outputs)| Append {
                inputs: inputs.try_into().unwrap(),
                outputs: outputs.try_into().unwrap(),
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
    pub commitment: [Target; 4],
    pub fields: [Target; Note::FIELD_SIZE],
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

        // Create input and output atoms
        for _ in 0..I {
            let (commitment, fields) = circuit_seal_note(&mut builder);
            inputs.push(Atom {
                commitment: commitment.elements,
                fields: fields.try_into().unwrap(),
            });
        }

        for _ in 0..O {
            let (commitment, fields) = circuit_seal_note(&mut builder);
            outputs.push(Atom {
                commitment: commitment.elements,
                fields: fields.try_into().unwrap(),
            });
        }

        // For each unique asset_id/asset_name pair, verify sum of amounts matches
        for i in 0..I {
            let input = &inputs[i];
            let input_amount = input.fields[0]; // First field is amount
            let input_asset_id = &input.fields[1..5]; // Next 4 fields are asset_id
            let input_asset_name = &input.fields[5..9]; // Next 4 fields are asset_name

            // Sum up all input amounts for this asset
            let mut input_sum = input_amount;
            for j in (i + 1)..I {
                let other = &inputs[j];
                let other_amount = other.fields[0];
                let other_asset_id = &other.fields[1..5];
                let other_asset_name = &other.fields[5..9];

                // Check if assets match
                let mut assets_match = builder._true();
                for k in 0..4 {
                    let id_match = builder.is_equal(input_asset_id[k], other_asset_id[k]);
                    let name_match = builder.is_equal(input_asset_name[k], other_asset_name[k]);
                    let both_match = builder.and(id_match, name_match);
                    assets_match = builder.and(assets_match, both_match);
                }

                // Add amount if assets match
                let should_add = builder.mul(assets_match.target, other_amount);
                input_sum = builder.add(input_sum, should_add);
            }

            // Sum up all output amounts for this asset
            let mut output_sum = builder.zero();

            for output in &outputs {
                let output_amount = output.fields[0];
                let output_asset_id = &output.fields[1..5];
                let output_asset_name = &output.fields[5..9];

                // Check if assets match
                let mut assets_match = builder._true();
                for k in 0..4 {
                    let id_match = builder.is_equal(input_asset_id[k], output_asset_id[k]);
                    let name_match = builder.is_equal(input_asset_name[k], output_asset_name[k]);
                    let both_match = builder.and(id_match, name_match);
                    assets_match = builder.and(assets_match, both_match);
                }

                // Add amount if assets match
                let should_add = builder.mul(assets_match.target, output_amount);
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
            // Set commitment
            let commitment = PoseidonHash::hash_no_pad(&input.note.as_fields_with_prefix());

            for (j, element) in commitment.elements.iter().enumerate() {
                pw.set_target(circuit.inputs[i].commitment[j], *element);
            }

            // Set fields
            for (j, field) in input.note.as_fields_with_prefix().iter().enumerate() {
                pw.set_target(circuit.inputs[i].fields[j], *field);
            }
        }

        // Set output values
        for (i, output) in self.outputs.iter().enumerate() {
            // Set commitment
            let commitment = PoseidonHash::hash_no_pad(&output.as_fields_with_prefix());

            for (j, element) in commitment.elements.iter().enumerate() {
                pw.set_target(circuit.outputs[i].commitment[j], *element);
            }

            // Set fields
            for (j, field) in output.as_fields_with_prefix().iter().enumerate() {
                pw.set_target(circuit.outputs[i].fields[j], *field);
            }
        }

        unwind_panic!(circuit.data.prove(pw).map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        }))
    }
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
