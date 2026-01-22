use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{AssetName, BlindSignature, PolicyId, Refresh};

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "p")]
pub enum Request {
    #[serde(rename = "refresh")]
    Refresh(Refresh),
    #[serde(rename = "emit")]
    Emit {
        policy_id: PolicyId,
        asset_name: AssetName,
        amount: u64,
    },
    #[serde(rename = "public_key")]
    Info,
    #[serde(rename = "deposit")]
    Deposit(DepositRequest),
    #[serde(rename = "withdraw")]
    Withdraw(WithdrawRequest),
}

/// Deposit request from user
/// User sends funds to script address and provides proof of deposit
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct DepositRequest {
    /// UTxO reference (tx_hash + index) at the script address
    pub utxo: UtxoReference,
    /// Blinded outputs to mint
    pub outputs: Vec<BlindSignature>,
    /// Message signed by user (canonical JSON)
    pub message: String,
    /// CIP-8 signature over canonical payload
    pub signature: Vec<u8>,
    /// Nonce/timestamp to prevent replay
    pub nonce: u64,
    /// Network tag (mainnet/preprod/etc)
    pub network: String,
}

/// UTxO reference for deposits
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct UtxoReference {
    /// Transaction hash (hex encoded)
    pub tx_hash: String,
    /// Output index
    pub index: u16,
}

/// Withdrawal request from user
/// User provides unsigned transaction spending script UTxOs
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct WithdrawRequest {
    /// Notes to burn (blinded inputs)
    pub notes: Vec<BlindSignature>,
    /// Unsigned transaction CBOR (hex encoded)
    pub tx_cbor: String,
    /// Transaction hash (expected)
    pub tx_hash: String,
}

/// Deposit response
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct DepositResponse {
    /// Blind signatures for the outputs
    pub signatures: Vec<BlindSignature>,
    /// Deposit reference (UTxO identifier)
    pub deposit_ref: String,
}

/// Withdrawal response
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct WithdrawResponse {
    /// Fully signed transaction CBOR (hex encoded)
    pub signed_tx_cbor: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Change notes (if any)
    pub change_notes: Vec<BlindSignature>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::types::{Refresh, Request};

    #[test]
    fn test_serialization() {
        let request: Request = Request::Refresh(Refresh::default());

        let expected = json!({
            "m": "refresh",
            "p": {
                "m": 0,
                "a": [],
                "a_": [],
                "s": [],
            }
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }

    #[test]
    fn test_info_serialization() {
        let request: Request = Request::Info;
        let expected = json!({
            "m": "public_key"
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }
}
