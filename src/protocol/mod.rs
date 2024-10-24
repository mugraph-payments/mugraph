mod codec;

mod message;
mod note;
mod seal;
mod types;

pub use self::{
    codec::*,
    message::*,
    note::{Note, SealedNote},
    seal::*,
    types::*,
};
use crate::Error;

pub const D: usize = 2;

pub type C = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type F = <C as plonky2::plonk::config::GenericConfig<D>>::F;
pub type Proof = plonky2::plonk::proof::ProofWithPublicInputs<F, C, D>;
pub type CircuitData = plonky2::plonk::circuit_data::CircuitData<F, C, D>;
pub type CircuitBuilder = plonky2::plonk::circuit_builder::CircuitBuilder<F, D>;
pub type Seal = plonky2::plonk::proof::CompressedProof<F, C, D>;
pub type CircuitConfig = plonky2::plonk::circuit_data::CircuitConfig;
pub type PartialWitness = plonky2::iop::witness::PartialWitness<F>;

use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};
pub use plonky2::{
    field::types::{Field, PrimeField64},
    hash::poseidon::PoseidonHash,
    iop::witness::WitnessWrite,
    plonk::config::Hasher,
};
use plonky2::{hash::hash_types::HashOutTarget, iop::target::Target};

pub(crate) fn circuit_builder() -> CircuitBuilder {
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

/// Converts a note's fields into a curve point for signing.
///
/// This function is used before blinding to get the initial note point.
/// It hashes the input message to a scalar and then multiplies it with
/// the base point to get a point on the curve.
///
/// # Arguments
///
/// * `message` - A byte slice containing the note's fields to be hashed.
///
/// # Returns
///
/// Returns an `RistrettoPoint` representing the hashed note on the curve.
pub fn hash_to_curve(note: &Note) -> Result<RistrettoPoint, Error> {
    let hash: Hash = PoseidonHash::hash_no_pad(&note.as_fields()).into();

    CompressedRistretto::from_slice(&hash.inner())
        .map_err(|e| Error::DecodeError(e.to_string()))?
        .decompress()
        .ok_or(Error::DecodeError("Failed to decompress hash".to_string()))
}

pub(crate) fn circuit_hash_to_curve(builder: &mut CircuitBuilder, data: &[Target]) -> HashOutTarget {
    let prefix = magic_prefix();
    let t0 = builder.constant(prefix[0]);
    let t1 = builder.constant(prefix[1]);

    builder.hash_n_to_hash_no_pad::<PoseidonHash>([&[t0, t1], data].concat())
}
