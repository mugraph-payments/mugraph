mod codec;

mod message;
pub mod note;
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

// `mu` in hex
pub const MAGIC_NUMBER: u16 = 0x6D75;

pub(crate) fn circuit_builder() -> CircuitBuilder {
    let config = CircuitConfig::standard_recursion_config();
    CircuitBuilder::new(config)
}
