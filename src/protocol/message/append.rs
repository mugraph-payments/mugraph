use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{protocol::*, Error};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Arbitrary)]
pub struct Append {
    pub inputs: Vec<Note>,
    pub outputs: Vec<Note>,
}

pub struct Circuit {
    data: CircuitData,
}

impl Sealable for Append {
    type Circuit = Circuit;

    fn circuit() -> Self::Circuit {
        todo!()
    }

    fn circuit_data() -> CircuitData {
        Self::circuit().data
    }

    fn prove(&self) -> Result<Proof, Error> {
        todo!()
    }
}

impl EncodeFields for Append {
    fn as_fields(&self) -> Vec<F> {
        let mut fields = Vec::new();

        // Add number of inputs and outputs
        fields.push(F::from_canonical_u64(self.inputs.len() as u64));
        fields.push(F::from_canonical_u64(self.outputs.len() as u64));

        // Encode inputs
        for input in &self.inputs {
            fields.extend(input.as_fields());
        }

        // Encode outputs
        for output in &self.outputs {
            fields.extend(output.as_fields());
        }

        fields
    }
}

impl DecodeFields for Append {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() < 2 {
            return Err(Error::DecodeError(
                "Not enough fields for Append".to_string(),
            ));
        }

        let num_inputs = fields[0].to_canonical_u64() as usize;
        let num_outputs = fields[1].to_canonical_u64() as usize;
        let mut cursor = 2;

        let mut inputs = Vec::with_capacity(num_inputs);
        let mut outputs = Vec::with_capacity(num_outputs);

        for _ in 0..num_inputs {
            if cursor + Note::FIELD_SIZE > fields.len() {
                return Err(Error::DecodeError(
                    "Not enough fields for input note".to_string(),
                ));
            }
            inputs.push(Note::from_fields(
                &fields[cursor..cursor + Note::FIELD_SIZE],
            )?);
            cursor += Note::FIELD_SIZE;
        }

        for _ in 0..num_outputs {
            if cursor + Note::FIELD_SIZE > fields.len() {
                return Err(Error::DecodeError(
                    "Not enough fields for output note".to_string(),
                ));
            }
            outputs.push(Note::from_fields(
                &fields[cursor..cursor + Note::FIELD_SIZE],
            )?);
            cursor += Note::FIELD_SIZE;
        }

        Ok(Append { inputs, outputs })
    }
}

#[cfg(test)]
mod tests {
    use super::Append;

    crate::test_encode_fields!(Append);
}
