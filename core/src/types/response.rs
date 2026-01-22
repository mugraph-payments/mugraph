use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "r")]
pub enum Response {
    #[serde(rename = "refresh")]
    Transaction {
        #[serde(rename = "s")]
        outputs: Vec<BlindSignature>,
    },
    #[serde(rename = "public_key")]
    Info {
        /// Node delegate public key
        delegate_pk: PublicKey,
        /// Cardano script address for deposits
        cardano_script_address: Option<String>,
    },
    #[serde(rename = "emit")]
    Emit(Box<Note>),
    #[serde(rename = "deposit")]
    Deposit {
        /// Blind signatures for the outputs
        #[serde(rename = "s")]
        signatures: Vec<BlindSignature>,
        /// Deposit reference (UTxO identifier: tx_hash:index)
        deposit_ref: String,
    },
    #[serde(rename = "withdraw")]
    Withdraw {
        /// Fully signed transaction CBOR (hex encoded)
        signed_tx_cbor: String,
        /// Transaction hash
        tx_hash: String,
        /// Change notes (if any)
        #[serde(rename = "s")]
        change_notes: Vec<BlindSignature>,
    },
    #[serde(rename = "error")]
    Error { reason: String },
}
