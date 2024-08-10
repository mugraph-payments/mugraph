use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use super::Signature;

#[derive(Serialize, Deserialize)]
pub enum Response {
    Success { outputs: Vec<Signature> },
    Error { message: String },
}
