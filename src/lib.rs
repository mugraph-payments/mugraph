#![deny(
    clippy::correctness,
    clippy::complexity,
    clippy::perf,
    clippy::big_endian_bytes
)]
#![warn(clippy::missing_inline_in_public_items)]

mod error;

pub mod protocol;
pub(crate) mod testing;
pub(crate) mod wallet;

use std::panic::UnwindSafe;

pub use self::{
    error::Error,
};
