use crate::{protocol::*, unwind_panic};

pub struct Append<const I: usize, const O: usize> {
    pub inputs: [SealedNote; I],
    pub outputs: [Note; O],
}

use std::collections::HashMap;

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
            let commitment = PoseidonHash::hash_no_pad(&input.note.as_fields());
            for (j, element) in commitment.elements.iter().enumerate() {
                pw.set_target(circuit.inputs[i].commitment[j], *element);
            }

            // Set fields
            for (j, field) in input.note.as_fields().iter().enumerate() {
                pw.set_target(circuit.inputs[i].fields[j], *field);
            }
        }

        // Set output values
        for (i, output) in self.outputs.iter().enumerate() {
            // Set commitment
            let commitment = PoseidonHash::hash_no_pad(&output.as_fields());
            for (j, element) in commitment.elements.iter().enumerate() {
                pw.set_target(circuit.outputs[i].commitment[j], *element);
            }

            // Set fields
            for (j, field) in output.as_fields().iter().enumerate() {
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
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;
}
