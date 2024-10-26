use std::panic::RefUnwindSafe;

use plonky2::plonk::proof::CompressedProofWithPublicInputs;

use super::*;
use crate::{unwind_panic, Error};

pub trait Sealable: EncodeFields + RefUnwindSafe {
    type Circuit;
    type Payload: EncodeFields;

    fn circuit() -> Self::Circuit;
    fn circuit_data() -> CircuitData;
    fn prove(&self) -> Result<Proof, Error>;

    fn seal(&self) -> Result<Seal, Error> {
        let compressed = unwind_panic(move || {
            let data = Self::circuit_data();
            let proof = self.prove()?;

            proof
                .compress(&data.verifier_only.circuit_digest, &data.common)
                .map_err(Error::from)
        })?;

        Ok(compressed.proof)
    }

    fn verify(payload: Self::Payload, proof: Seal) -> Result<(), Error> {
        let proof = CompressedProofWithPublicInputs {
            proof,
            public_inputs: payload.as_fields(),
        };

        unwind_panic(|| {
            Self::circuit_data()
                .verify_compressed(proof)
                .map_err(Error::from)
        })
    }
}
