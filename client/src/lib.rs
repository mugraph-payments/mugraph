use std::collections::HashMap;

use mugraph_core::types::{Hash, Note};

mod builder;

pub mod prelude {
    pub use mugraph_core::{
        error::{Error, Result},
        types::{Blob, Hash, Manifest, Note, ProgramSet, Transaction, MAX_ATOMS, MAX_INPUTS},
    };
    pub use mugraph_core_programs::__build::{VALIDATE_ELF, VALIDATE_ID};

    pub use crate::builder::TransactionBuilder;
}

pub struct Wallet {
    pub balance: HashMap<Hash, u64>,
    pub notes: Vec<Note>,
}
