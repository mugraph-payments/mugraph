use plonky2::plonk::proof::CompressedProofWithPublicInputs;

use super::*;
use crate::{unwind_panic, Error};

pub trait Sealable: EncodeFields {
    type Circuit;

    fn circuit() -> Self::Circuit;
    fn circuit_data() -> CircuitData;
    fn prove(&self) -> Result<Proof, Error>;

    fn seal(&self) -> Result<Seal, Error> {
        let data = Self::circuit_data();
        let proof = unwind_panic!(self.prove())?;

        let proof = unwind_panic!(proof.compress(&data.verifier_only.circuit_digest, &data.common))
            .map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        })?;

        Ok(proof.proof)
    }

    fn verify(hash: Hash, proof: Seal) -> Result<(), Error> {
        let proof = CompressedProofWithPublicInputs {
            proof,
            public_inputs: hash.as_fields(),
        };

        unwind_panic!(Self::circuit_data().verify_compressed(proof)).map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        })
    }
}
