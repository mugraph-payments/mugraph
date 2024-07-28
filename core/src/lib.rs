#![no_std]

pub mod crypto;
mod error;
mod event;
mod hash;
mod note;
mod operation;
mod signature;

pub use self::{error::*, event::*, hash::*, note::*, operation::*, signature::*};

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;
pub type SecretKey = Scalar;
pub type PublicKey = Point;

pub const OUTPUT_SEP: Hash = Hash::new([
    251, 27, 10, 119, 219, 137, 49, 221, 246, 211, 108, 158, 213, 143, 56, 34, 184, 84, 252, 192,
    213, 154, 116, 137, 200, 235, 231, 113, 178, 201, 48, 84,
]);
pub const CHANGE_SEP: Hash = Hash::new([
    79, 137, 80, 88, 98, 115, 151, 241, 192, 91, 151, 240, 66, 7, 83, 47, 252, 9, 195, 57, 84, 201,
    158, 76, 251, 117, 116, 203, 34, 242, 57, 247,
]);

pub const DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_HASH_TO_CURVE_";
pub const DLEQ_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_DLEQ_PROOF_";
pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
