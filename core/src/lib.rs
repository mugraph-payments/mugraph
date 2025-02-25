use std::sync::{Arc, atomic::AtomicBool};

pub mod builder;
pub mod crypto;
pub mod error;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod testing;

pub type Signal = Arc<AtomicBool>;
