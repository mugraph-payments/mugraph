use std::panic::RefUnwindSafe;

use plonky2::plonk::proof::CompressedProofWithPublicInputs;
pub use plonky2::{
    field::types::{Field, PrimeField64},
    hash::{hash_types::HashOutTarget, poseidon::PoseidonHash},
    iop::{
        target::{BoolTarget, Target},
        witness::WitnessWrite,
    },
    plonk::config::Hasher,
};

use super::*;
use crate::{unwind_panic, Error};

pub const D: usize = 2;

pub type C = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type F = <C as plonky2::plonk::config::GenericConfig<D>>::F;
pub type Proof = plonky2::plonk::proof::ProofWithPublicInputs<F, C, D>;
pub type CircuitData = plonky2::plonk::circuit_data::CircuitData<F, C, D>;
pub type CircuitBuilder = plonky2::plonk::circuit_builder::CircuitBuilder<F, D>;
pub type Seal = plonky2::plonk::proof::CompressedProof<F, C, D>;
pub type CircuitConfig = plonky2::plonk::circuit_data::CircuitConfig;
pub type PartialWitness = plonky2::iop::witness::PartialWitness<F>;

#[inline(always)]
pub fn circuit_builder() -> CircuitBuilder {
    let config = CircuitConfig::standard_recursion_config();
    CircuitBuilder::new(config)
}

#[inline(always)]
pub fn magic_prefix() -> [F; 2] {
    [
        F::from_canonical_u64(MAGIC_PREFIX_FIELDS[0]),
        F::from_canonical_u64(MAGIC_PREFIX_FIELDS[1]),
    ]
}

#[inline]
pub fn seal_note(builder: &mut CircuitBuilder) -> (HashOutTarget, Vec<Target>) {
    let zero = builder.zero();
    let note_size = Note::default().field_len();

    // Private inputs
    let mut targets = builder.add_virtual_targets(note_size);
    let (amount, rest) = targets.split_at_mut(1);
    let (asset_id, rest) = rest.split_at_mut(4);
    let (asset_name, nonce) = rest.split_at_mut(4);

    // Public input
    let commitment = hash_to_curve(
        builder,
        &[
            amount.to_vec(),
            asset_id.to_vec(),
            asset_name.to_vec(),
            nonce.to_vec(),
        ]
        .concat(),
    );

    // Assert amount is non-zero
    let is_zero = builder.is_equal(amount[0], zero);
    builder.assert_zero(is_zero.target);

    let asset_id_not_zero = targets_are_zero(builder, asset_id);
    builder.assert_zero(asset_id_not_zero.target);
    let asset_name_not_zero = targets_are_zero(builder, asset_name);
    builder.assert_zero(asset_name_not_zero.target);
    let nonce_not_zero = targets_are_zero(builder, nonce);
    builder.assert_zero(nonce_not_zero.target);

    (commitment, targets)
}

#[inline]
pub fn hash_to_curve(builder: &mut CircuitBuilder, data: &[Target]) -> HashOutTarget {
    let [t0, t1] = magic_prefix();
    let t0 = builder.constant(t0);
    let t1 = builder.constant(t1);

    builder.hash_n_to_hash_no_pad::<PoseidonHash>([&[t0, t1], data].concat())
}

fn targets_are_zero(builder: &mut CircuitBuilder, targets: &[Target]) -> BoolTarget {
    let zero = builder.zero();
    let mut target = builder._true();

    for input in targets {
        let is_zero = builder.is_equal(*input, zero);
        target = builder.and(target, is_zero);
    }

    target
}

pub trait Sealable: Encode + RefUnwindSafe {
    type Circuit;
    type Payload: Encode;

    fn circuit() -> Self::Circuit;
    fn circuit_data() -> CircuitData;
    fn prove(&self) -> Result<Proof, Error>;

    #[inline]
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

    #[inline]
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
