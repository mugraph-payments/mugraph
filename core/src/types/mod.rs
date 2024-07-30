mod hash;
mod note;
mod signature;

pub use self::{hash::*, note::*, signature::*};

pub type SecretKey = [u8; 64];
pub type PublicKey = [u8; 32];
