mod codec;

pub mod crypto;
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

pub const D: usize = 2;

pub type C = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type F = <C as plonky2::plonk::config::GenericConfig<D>>::F;
pub type Proof = plonky2::plonk::proof::ProofWithPublicInputs<F, C, D>;
pub type CircuitData = plonky2::plonk::circuit_data::CircuitData<F, C, D>;
pub type CircuitBuilder = plonky2::plonk::circuit_builder::CircuitBuilder<F, D>;
pub type Seal = plonky2::plonk::proof::CompressedProof<F, C, D>;
pub type CircuitConfig = plonky2::plonk::circuit_data::CircuitConfig;
pub type PartialWitness = plonky2::iop::witness::PartialWitness<F>;

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

pub(crate) fn circuit_hash_to_curve(builder: &mut CircuitBuilder, data: &[Target]) -> HashOutTarget {
    let prefix = magic_prefix();
    let t0 = builder.constant(prefix[0]);
    let t1 = builder.constant(prefix[1]);

    builder.hash_n_to_hash_no_pad::<PoseidonHash>([&[t0, t1], data].concat())
}
