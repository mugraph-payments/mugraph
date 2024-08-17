use serde::{Deserialize, Serialize};

use super::{Blinded, Signature};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub outputs: Vec<Blinded<Signature>>,
}
