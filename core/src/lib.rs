#![cfg_attr(not(feature = "std"), no_std)]

pub mod contracts;
pub mod crypto;

mod error;
mod io;
mod serialize;
mod types;

pub use self::{error::*, io::*, serialize::*, types::*};

pub const OUTPUT_SEP: Hash = Hash::new([
    251, 27, 10, 119, 219, 137, 49, 221, 246, 211, 108, 158, 213, 143, 56, 34, 184, 84, 252, 192,
    213, 154, 116, 137, 200, 235, 231, 113, 178, 201, 48, 84,
]);
pub const CHANGE_SEP: Hash = Hash::new([
    79, 137, 80, 88, 98, 115, 151, 241, 192, 91, 151, 240, 66, 7, 83, 47, 252, 9, 195, 57, 84, 201,
    158, 76, 251, 117, 116, 203, 34, 242, 57, 247,
]);
pub const HTC_SEP: Hash = Hash::new([
    244, 129, 16, 184, 206, 78, 78, 149, 20, 45, 241, 229, 142, 175, 218, 14, 173, 29, 12, 6, 180,
    108, 3, 238, 41, 141, 212, 239, 112, 242, 238, 62,
]);

#[doc(hidden)]
/// This is a hidden module, re-exporting crate dependencies.
///
/// This is necessary so the macros can be used directly without having to import those
/// dependencies.
pub mod __dependencies {
    pub use paste::paste;
}
