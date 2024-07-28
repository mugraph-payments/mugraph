#![no_std]

mod error;
mod hash;
mod note;
mod operation;

pub use self::{error::*, hash::*, note::*, operation::*};

pub const OUTPUT_SEP: [u8; 14] = *b"MUGRAPH_OUTPUT";
pub const CHANGE_SEP: [u8; 14] = *b"MUGRAPH_CHANGE";
