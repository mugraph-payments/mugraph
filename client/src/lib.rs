use std::collections::HashMap;

use mugraph_core::types::{Hash, Note};

mod builder;

pub mod prelude {
    pub use mugraph_core::{
        crypto,
        error::{Error, Result},
        types::*,
    };

    pub use crate::builder::TransactionBuilder;
}

pub struct Wallet {
    pub balance: HashMap<Hash, u64>,
    pub notes: Vec<Note>,
}
