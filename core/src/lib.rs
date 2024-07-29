#![no_std]

mod error;
mod event;
mod hash;
mod note;
mod operation;
mod signature;

pub use self::{error::*, event::*, hash::*, note::*, operation::*, signature::*};

pub type PublicKey = [u8; 32];

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
