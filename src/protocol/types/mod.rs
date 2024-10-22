mod hash;
mod name;
mod public_key;
mod signature;

pub use self::{hash::Hash, name::*, public_key::*, signature::*};

// Ed25519 Public Key
pub type SecretKey = [u8; 32];
