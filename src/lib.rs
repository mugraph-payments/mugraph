#![deny(
    clippy::correctness,
    clippy::complexity,
    clippy::perf,
    clippy::big_endian_bytes
)]
#![warn(clippy::missing_inline_in_public_items)]

mod error;

pub mod crypto;
pub mod protocol;
pub(crate) mod testing;
pub(crate) mod wallet;

use std::panic::UnwindSafe;

pub use self::{
    error::Error,
    protocol::{Decode, DecodeFields, Encode, EncodeFields},
};

#[inline]
pub fn unwind_panic<T, F>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error> + UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(err)) => Err(err),
        Err(panic) => {
            let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };

            Err(Error::Panic(panic_msg))
        }
    }
}
