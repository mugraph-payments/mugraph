use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use super::{Blinded, Signature};

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Success { outputs: Vec<Blinded<Signature>> },
    Error { message: String },
}
