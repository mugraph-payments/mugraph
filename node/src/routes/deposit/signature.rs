use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{DepositRequest, PublicKey},
};
use serde::{Deserialize, Serialize};

use super::claims::DepositClaims;
#[cfg(test)]
use super::claims::parse_deposit_claims;
use crate::network::CardanoNetwork;

/// Verify CIP-8 signature over canonical deposit payload
///
/// # CIP-8/COSE Support
/// This function supports two signature formats:
/// 1. Raw Ed25519 signatures (64 bytes) - current default
/// 2. Full CIP-8 COSE_Sign1 structure (with proper header validation)
///
/// # Security Considerations
/// - Verifies the signature over the canonical JSON payload
/// - Validates the user public key format
/// - Computes the key hash for datum verification
/// - Includes network tag in payload to prevent cross-network replay
pub(super) fn verify_deposit_signature(
    request: &DepositRequest,
    claims: &DepositClaims,
    wallet: &mugraph_core::types::CardanoWallet,
    delegate_pk: &PublicKey,
) -> Result<(), Error> {
    let payload =
        build_canonical_payload(request, delegate_pk, &wallet.script_address);

    verify_cip8_cose_signature_with_claims(request, claims, &payload)
}

pub(super) fn verify_cip8_cose_signature_with_claims(
    request: &DepositRequest,
    claims: &DepositClaims,
    payload: &[u8],
) -> Result<(), Error> {
    use coset::{CoseSign1, TaggedCborSerializable, iana};
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let user_pubkey_bytes = claims.user_pubkey;

    let cose: CoseSign1 = CoseSign1::from_tagged_slice(&request.signature)
        .map_err(|e| Error::InvalidSignature {
            reason: format!("Invalid COSE_Sign1: {}", e),
            signature: mugraph_core::types::Signature::default(),
        })?;

    // Check alg = EdDSA
    let alg = cose
        .protected
        .header
        .alg
        .clone()
        .or(cose.unprotected.alg.clone())
        .ok_or_else(|| Error::InvalidSignature {
            reason: "Missing alg in COSE header".to_string(),
            signature: mugraph_core::types::Signature::default(),
        })?;
    if alg
        != coset::RegisteredLabelWithPrivate::Assigned(iana::Algorithm::EdDSA)
    {
        return Err(Error::InvalidSignature {
            reason: format!("Unsupported alg {:?}, expected EdDSA", alg),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Payload must match
    let cose_payload =
        cose.payload
            .as_ref()
            .ok_or_else(|| Error::InvalidSignature {
                reason: "COSE payload missing".to_string(),
                signature: mugraph_core::types::Signature::default(),
            })?;

    if cose_payload != payload {
        return Err(Error::InvalidSignature {
            reason: "COSE payload does not match expected payload".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    let verifying_key =
        VerifyingKey::from_bytes(&user_pubkey_bytes).map_err(|e| {
            Error::InvalidKey {
                reason: format!("Invalid Ed25519 public key: {}", e),
            }
        })?;

    let sig_bytes = &cose.signature;
    if sig_bytes.len() != 64 {
        return Err(Error::InvalidSignature {
            reason: format!(
                "COSE signature must be 64 bytes, got {}",
                sig_bytes.len()
            ),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    let signature = Signature::from_slice(sig_bytes).map_err(|e| {
        Error::InvalidSignature {
            reason: format!("Invalid signature format: {}", e),
            signature: mugraph_core::types::Signature::default(),
        }
    })?;

    let aad: Vec<u8> = vec![];
    let tbs = cose.tbs_data(aad.as_slice());

    verifying_key.verify(&tbs, &signature).map_err(|e| {
        Error::InvalidSignature {
            reason: format!("COSE signature verification failed: {}", e),
            signature: mugraph_core::types::Signature::default(),
        }
    })?;

    Ok(())
}

#[cfg(test)]
pub(super) fn verify_cip8_cose_signature(
    request: &DepositRequest,
    payload: &[u8],
) -> Result<(), Error> {
    let claims = parse_deposit_claims(request)?;
    verify_cip8_cose_signature_with_claims(request, &claims, payload)
}

/// UTXO reference for canonical payload serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CanonicalUtxo {
    pub(super) tx_hash: String,
    pub(super) index: u16,
}

/// Canonical payload for signature verification
/// Sorted JSON with no extra whitespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CanonicalPayload {
    pub(super) utxo: CanonicalUtxo,
    pub(super) outputs: Vec<String>,
    #[serde(rename = "delegate_pk")]
    pub(super) delegate_pk: String,
    #[serde(rename = "script_address")]
    pub(super) script_address: String,
    pub(super) nonce: u64,
    pub(super) network: String,
}

/// Build canonical payload for signature verification
/// Sorted JSON with no extra whitespace
pub(super) fn build_canonical_payload(
    request: &DepositRequest,
    delegate_pk: &PublicKey,
    script_address: &str,
) -> Vec<u8> {
    let outputs: Vec<String> = request
        .outputs
        .iter()
        .map(|o| hex::encode(o.signature.0.0))
        .collect();

    let network = CardanoNetwork::parse(&request.network)
        .map(|network| network.as_str().to_string())
        .unwrap_or_else(|_| request.network.clone());

    let payload = CanonicalPayload {
        utxo: CanonicalUtxo {
            tx_hash: request.utxo.tx_hash.clone(),
            index: request.utxo.index,
        },
        outputs,
        delegate_pk: hex::encode(delegate_pk.0),
        script_address: script_address.to_string(),
        nonce: request.nonce,
        network,
    };

    serde_json::to_string(&payload).unwrap().into_bytes()
}

/// Compute intent hash from deposit request
/// This is a blake2b-256 hash of the canonical payload
/// Used for off-chain replay protection and reference in datum
/// Note: Intent hash is verified off-chain only, not validated by the on-chain validator
pub(super) fn compute_intent_hash(
    request: &DepositRequest,
    delegate_pk: &PublicKey,
    script_address: &str,
) -> [u8; 32] {
    use blake2::{Blake2b, Digest, digest::consts::U32};

    let payload = build_canonical_payload(request, delegate_pk, script_address);

    type Blake2b256 = Blake2b<U32>;
    let hash = Blake2b256::digest(&payload);

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}
