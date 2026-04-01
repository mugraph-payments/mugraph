use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::DepositRequest};

pub(super) struct DepositClaims {
    pub(super) user_pubkey: [u8; 32],
}

pub(super) fn parse_deposit_claims(
    request: &DepositRequest,
) -> Result<DepositClaims, Error> {
    let message_json: serde_json::Value =
        serde_json::from_str(&request.message).map_err(|e| {
            Error::InvalidInput {
                reason: format!("Invalid message JSON: {}", e),
            }
        })?;
    let user_pubkey_hex = message_json
        .get("user_pubkey")
        .and_then(|value| value.as_str())
        .ok_or_else(|| Error::InvalidInput {
            reason: "Missing user_pubkey in message".to_string(),
        })?;
    let user_pubkey_bytes =
        hex::decode(user_pubkey_hex).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid user_pubkey hex: {}", e),
        })?;
    let user_pubkey_len = user_pubkey_bytes.len();
    let user_pubkey: [u8; 32] =
        user_pubkey_bytes
            .try_into()
            .map_err(|_| Error::InvalidInput {
                reason: format!(
                    "user_pubkey must be 32 bytes, got {}",
                    user_pubkey_len
                ),
            })?;

    Ok(DepositClaims { user_pubkey })
}
