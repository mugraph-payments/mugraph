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

pub const G: RistrettoPoint = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
pub type C = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type F = <C as plonky2::plonk::config::GenericConfig<D>>::F;
pub type Proof = plonky2::plonk::proof::ProofWithPublicInputs<F, C, D>;
pub type CircuitData = plonky2::plonk::circuit_data::CircuitData<F, C, D>;
pub type CircuitBuilder = plonky2::plonk::circuit_builder::CircuitBuilder<F, D>;
pub type Seal = plonky2::plonk::proof::CompressedProof<F, C, D>;
pub type CircuitConfig = plonky2::plonk::circuit_data::CircuitConfig;
pub type PartialWitness = plonky2::iop::witness::PartialWitness<F>;

use curve25519_dalek::{RistrettoPoint, Scalar};
pub use plonky2::{
    field::types::{Field, PrimeField64},
    hash::poseidon::PoseidonHash,
    iop::witness::WitnessWrite,
    plonk::config::Hasher,
};
use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
};

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
    let hash: Hash = PoseidonHash::hash_no_pad(&note.as_fields_with_prefix()).into();
    let res = Scalar::from_bytes_mod_order(hash.as_bytes().try_into().unwrap());

    Ok(res * G)
}

pub(crate) fn circuit_hash_to_curve(builder: &mut CircuitBuilder, data: &[Target]) -> HashOutTarget {
    let prefix = magic_prefix();
    let t0 = builder.constant(prefix[0]);
    let t1 = builder.constant(prefix[1]);

    builder.hash_n_to_hash_no_pad::<PoseidonHash>([&[t0, t1], data].concat())
}

pub(crate) fn circuit_seal_note(builder: &mut CircuitBuilder) -> (HashOutTarget, Vec<Target>) {
    let zero = builder.zero();

    // Private inputs
    let mut targets = builder.add_virtual_targets(Note::FIELD_SIZE);
    let (amount, rest) = targets.split_at_mut(1);
    let (asset_id, rest) = rest.split_at_mut(4);
    let (asset_name, nonce) = rest.split_at_mut(4);

    // Public input
    let commitment = circuit_hash_to_curve(
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

    let asset_id_not_zero = hash_is_zero(builder, asset_id);
    builder.assert_zero(asset_id_not_zero.target);
    let asset_name_not_zero = hash_is_zero(builder, asset_name);
    builder.assert_zero(asset_name_not_zero.target);
    let nonce_not_zero = hash_is_zero(builder, nonce);
    builder.assert_zero(nonce_not_zero.target);

    (commitment, targets)
}

fn hash_is_zero(builder: &mut CircuitBuilder, targets: &[Target]) -> BoolTarget {
    let zero = builder.zero();
    let mut target = builder._true();

    for input in targets {
        let is_zero = builder.is_equal(*input, zero);
        target = builder.and(target, is_zero);
    }

    target
}
