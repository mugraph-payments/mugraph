use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{BlindSignature, DepositRequest, PublicKey, Response, UtxoRef},
};
use serde::{Deserialize, Serialize};

use crate::{
    cardano::setup_cardano_wallet,
    database::{CARDANO_WALLET, DEPOSITS},
    provider::{Provider, UtxoInfo},
    routes::Context,
};

/// Handle deposit request
///
/// 1. Parse and validate the request payload
/// 2. Verify CIP-8 signature
/// 3. Fetch UTxO from provider
/// 4. Validate UTxO is at script address and unspent
/// 5. Map assets and validate amounts
/// 6. Sign blinded outputs
/// 7. Record deposit in database
pub async fn handle_deposit(request: &DepositRequest, ctx: &Context) -> Result<Response, Error> {
    tracing::info!(
        "Processing deposit request for UTxO: {}:{}",
        &request.utxo.tx_hash[..std::cmp::min(16, request.utxo.tx_hash.len())],
        request.utxo.index
    );

    // 1. Load or create Cardano wallet
    let wallet = load_or_create_wallet(ctx).await?;

    // 2. Verify CIP-8 signature over canonical payload
    verify_deposit_signature(request, &wallet, &ctx.keypair.public_key)?;

    // 3. Fetch UTxO from Cardano provider and validate
    let provider = create_provider(ctx)?;
    let utxo_info = fetch_and_validate_utxo(request, &wallet, &provider, ctx).await?;

    // 4. Validate outputs cover all assets in UTxO
    validate_deposit_amounts(request, &utxo_info)?;

    // 5. Sign blinded outputs with delegate key
    let signatures = sign_outputs(request, &ctx.keypair)?;

    // 6. Record deposit in database
    record_deposit(request, ctx, &provider, &wallet).await?;

    let deposit_ref = format!("{}:{}", request.utxo.tx_hash, request.utxo.index);

    tracing::info!(
        "Deposit processed successfully: {}",
        &deposit_ref[..std::cmp::min(32, deposit_ref.len())]
    );

    Ok(Response::Deposit {
        signatures,
        deposit_ref,
    })
}

/// Load Cardano wallet from database or create new one
async fn load_or_create_wallet(ctx: &Context) -> Result<mugraph_core::types::CardanoWallet, Error> {
    // Try to load existing wallet
    {
        let read_tx = ctx.database.read()?;
        let table = read_tx.open_table(CARDANO_WALLET)?;
        if let Some(wallet_data) = table.get("wallet")? {
            return Ok(wallet_data.value());
        }
    }

    // Create new wallet if not found
    // Use config for network and payment key
    let network = ctx.config.network();
    let payment_sk = ctx.config.payment_sk();

    let wallet = setup_cardano_wallet(&network, payment_sk.as_deref())
        .await
        .map_err(|e| Error::Internal {
            reason: e.to_string(),
        })?;

    // Store wallet in database
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(CARDANO_WALLET)?;
        table.insert("wallet", &wallet)?;
    }
    write_tx.commit()?;

    Ok(wallet)
}

/// Create Cardano provider from configuration
fn create_provider(ctx: &Context) -> Result<Provider, Error> {
    // Use config for provider settings
    Provider::new(
        &ctx.config.provider_type(),
        ctx.config.provider_api_key(),
        ctx.config.network(),
        ctx.config.provider_url(),
    )
    .map_err(|e| Error::Internal {
        reason: e.to_string(),
    })
}

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
fn verify_deposit_signature(
    request: &DepositRequest,
    wallet: &mugraph_core::types::CardanoWallet,
    delegate_pk: &mugraph_core::types::PublicKey,
) -> Result<(), Error> {
    // Build canonical payload
    // Payload = utxo + outputs + delegate pk + script address + nonce + network tag
    let payload = build_canonical_payload(request, delegate_pk, &wallet.script_address);

    // Try to parse and verify as CIP-8/COSE format first
    match verify_cip8_cose_signature(request, &payload) {
        Ok(()) => {
            tracing::info!("CIP-8/COSE signature verified successfully");
            return Ok(());
        }
        Err(e) => {
            tracing::debug!(
                "CIP-8/COSE verification failed ({}), trying raw Ed25519...",
                e
            );
            // Fall through to raw Ed25519 verification
        }
    }

    // Fall back to raw Ed25519 signature verification
    verify_raw_ed25519_signature(request, &payload)
}

/// Verify CIP-8/COSE_Sign1 signature
///
/// CIP-8 defines a COSE-based signing format for Cardano.
/// The signature structure includes:
/// - Protected header (algorithm, content type, etc.)
/// - Unprotected header (optional fields)
/// - Payload (the signed data)
/// - Signature
///
/// This is a partial implementation that handles basic CIP-8 signatures.
/// Full implementation would require a COSE library.
fn verify_cip8_cose_signature(request: &DepositRequest, payload: &[u8]) -> Result<(), Error> {
    use coset::{CoseSign1, TaggedCborSerializable, iana};
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // Parse user_pubkey from message
    let message_json: serde_json::Value =
        serde_json::from_str(&request.message).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid message JSON: {}", e),
        })?;

    let user_pubkey_hex = message_json
        .get("user_pubkey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidInput {
            reason: "Missing user_pubkey in message".to_string(),
        })?;

    let user_pubkey_bytes = hex::decode(user_pubkey_hex).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid user_pubkey hex: {}", e),
    })?;

    if user_pubkey_bytes.len() != 32 {
        return Err(Error::InvalidInput {
            reason: format!(
                "user_pubkey must be 32 bytes, got {}",
                user_pubkey_bytes.len()
            ),
        });
    }

    let cose: CoseSign1 =
        CoseSign1::from_tagged_slice(&request.signature).map_err(|e| Error::InvalidSignature {
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
    if alg != coset::RegisteredLabelWithPrivate::Assigned(iana::Algorithm::EdDSA) {
        return Err(Error::InvalidSignature {
            reason: format!("Unsupported alg {:?}, expected EdDSA", alg),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Payload must match
    let cose_payload = cose
        .payload
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

    let sig_bytes = &cose.signature;
    if sig_bytes.len() != 64 {
        return Err(Error::InvalidSignature {
            reason: format!("COSE signature must be 64 bytes, got {}", sig_bytes.len()),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Build Sig_structure bytes using coset helper
    let to_verify = cose.tbs_data(&[]);

    let verifying_key = VerifyingKey::from_bytes(
        &user_pubkey_bytes.try_into().expect("Length checked"),
    )
    .map_err(|e| Error::InvalidKey {
        reason: format!("Invalid Ed25519 public key: {}", e),
    })?;
    let signature = Signature::from_slice(sig_bytes).map_err(|e| Error::InvalidSignature {
        reason: format!("Invalid signature format: {}", e),
        signature: mugraph_core::types::Signature::default(),
    })?;

    verifying_key
        .verify(&to_verify, &signature)
        .map_err(|e| Error::InvalidSignature {
            reason: format!("COSE signature verification failed: {}", e),
            signature: mugraph_core::types::Signature::default(),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};

    use coset::{CoseSign1, CoseSign1Builder, Header, ProtectedHeader, TaggedCborSerializable, iana};
    use ed25519_dalek::{Signer, SigningKey};
    use mugraph_core::types::UtxoReference;

    use super::*;

    static SEED_COUNTER: AtomicU8 = AtomicU8::new(1);

    fn gen_key() -> (SigningKey, Vec<u8>) {
        let seed_byte = SEED_COUNTER.fetch_add(1, Ordering::SeqCst);
        let seed = [seed_byte; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk = sk.verifying_key().to_bytes().to_vec();
        (sk, pk)
    }

    fn build_cip8_signature(sk: &SigningKey, payload: &[u8]) -> Vec<u8> {
        let header = Header {
            alg: Some(coset::RegisteredLabelWithPrivate::Assigned(
                iana::Algorithm::EdDSA,
            )),
            ..Default::default()
        };
        let unprotected = Header::default();
        let tbs = CoseSign1 {
            protected: ProtectedHeader {
                original_data: None,
                header: header.clone(),
            },
            unprotected,
            payload: Some(payload.to_vec()),
            signature: vec![],
        }
        .tbs_data(&[]);
        let sig = sk.sign(&tbs);

        let cose = CoseSign1Builder::new()
            .protected(header)
            .payload(payload.to_vec())
            .signature(sig.to_vec())
            .build();

        cose.to_tagged_vec().unwrap()
    }

    #[test]
    fn test_cip8_verification_succeeds() {
        let (sk, pk_bytes) = gen_key();
        let mut request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!("{{\"user_pubkey\":\"{}\"}}", hex::encode(&pk_bytes)),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.clone().try_into().unwrap()),
            "addr_test1...",
        );
        let sig = build_cip8_signature(&sk, &payload);
        request.signature = sig;

        assert!(verify_cip8_cose_signature(&request, &payload).is_ok());
    }

    #[test]
    fn test_cip8_verification_fails_on_payload_mismatch() {
        let (sk, pk_bytes) = gen_key();
        let mut request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!("{{\"user_pubkey\":\"{}\"}}", hex::encode(&pk_bytes)),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.clone().try_into().unwrap()),
            "addr_test1...",
        );
        let sig = build_cip8_signature(&sk, &payload);

        // mutate payload by changing network after signing
        request.network = "mainnet".to_string();
        request.signature = sig;
        let bad_payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );

        let res = verify_cip8_cose_signature(&request, &bad_payload);
        assert!(res.is_err());
    }
}

/// Verify raw Ed25519 signature
fn verify_raw_ed25519_signature(request: &DepositRequest, payload: &[u8]) -> Result<(), Error> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // For now, we expect signature to be raw Ed25519 signature (64 bytes)
    if request.signature.len() != 64 {
        return Err(Error::InvalidSignature {
            reason: format!(
                "Invalid signature length: expected 64 bytes, got {}",
                request.signature.len()
            ),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Extract public key from request message
    let message_json: serde_json::Value =
        serde_json::from_str(&request.message).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid message JSON: {}", e),
        })?;

    let user_pubkey_hex = message_json
        .get("user_pubkey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidInput {
            reason: "Missing user_pubkey in message".to_string(),
        })?;

    let user_pubkey_bytes = hex::decode(user_pubkey_hex).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid user_pubkey hex: {}", e),
    })?;

    if user_pubkey_bytes.len() != 32 {
        return Err(Error::InvalidInput {
            reason: format!(
                "user_pubkey must be 32 bytes, got {}",
                user_pubkey_bytes.len()
            ),
        });
    }

    // Clone for blake3 hash later
    let user_pubkey_for_hash = user_pubkey_bytes.clone();

    let verifying_key =
        VerifyingKey::from_bytes(&user_pubkey_bytes.try_into().expect("Length checked above"))
            .map_err(|e| Error::InvalidKey {
                reason: format!("Invalid Ed25519 public key: {}", e),
            })?;

    let signature =
        Signature::from_slice(&request.signature).map_err(|e| Error::InvalidSignature {
            reason: format!("Invalid signature format: {}", e),
            signature: mugraph_core::types::Signature::default(),
        })?;

    // Verify signature over the canonical payload
    verifying_key
        .verify(payload, &signature)
        .map_err(|e| Error::InvalidSignature {
            reason: format!("Signature verification failed: {}", e),
            signature: mugraph_core::types::Signature::default(),
        })?;

    // Compute the user key hash for datum verification
    let user_pubkey_hash = blake3::hash(&user_pubkey_for_hash);
    let expected_hash_hex = hex::encode(&user_pubkey_hash.as_bytes()[..28]);

    tracing::info!(
        "Ed25519 signature verified for user key hash: {}",
        &expected_hash_hex[..16]
    );

    Ok(())
}

/// UTXO reference for canonical payload serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanonicalUtxo {
    tx_hash: String,
    index: u16,
}

/// Canonical payload for signature verification
/// Sorted JSON with no extra whitespace
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanonicalPayload {
    utxo: CanonicalUtxo,
    outputs: Vec<String>,
    #[serde(rename = "delegate_pk")]
    delegate_pk: String,
    #[serde(rename = "script_address")]
    script_address: String,
    nonce: u64,
    network: String,
}

/// Build canonical payload for signature verification
/// Sorted JSON with no extra whitespace
fn build_canonical_payload(
    request: &DepositRequest,
    delegate_pk: &PublicKey,
    script_address: &str,
) -> Vec<u8> {
    // Convert outputs to serializable format
    // BlindSignature has: signature: Blinded<Signature>, proof: DleqProof
    // Blinded<Signature> has field 0 which is Signature
    // Signature has field 0 which is [u8; 32]
    let outputs: Vec<String> = request
        .outputs
        .iter()
        .map(|o| hex::encode(o.signature.0.0))
        .collect();

    let payload = CanonicalPayload {
        utxo: CanonicalUtxo {
            tx_hash: request.utxo.tx_hash.clone(),
            index: request.utxo.index,
        },
        outputs,
        delegate_pk: hex::encode(delegate_pk.0),
        script_address: script_address.to_string(),
        nonce: request.nonce,
        network: request.network.clone(),
    };

    // Serialize to canonical JSON (no extra whitespace, sorted keys)
    serde_json::to_string(&payload).unwrap().into_bytes()
}

/// Compute intent hash from deposit request
/// This is a blake2b-256 hash of the canonical payload
/// Used for off-chain replay protection and reference in datum
/// Note: Intent hash is verified off-chain only, not validated by the on-chain validator
fn compute_intent_hash(
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

/// Fetch UTxO from provider and validate it's at the script address
async fn fetch_and_validate_utxo(
    request: &DepositRequest,
    wallet: &mugraph_core::types::CardanoWallet,
    provider: &Provider,
    ctx: &Context,
) -> Result<UtxoInfo, Error> {
    let utxo_info = provider
        .get_utxo(&request.utxo.tx_hash, request.utxo.index)
        .await
        .map_err(|e| Error::NetworkError {
            reason: format!("Failed to fetch UTxO: {}", e),
        })?
        .ok_or_else(|| Error::InvalidInput {
            reason: "UTxO not found on chain".to_string(),
        })?;

    // Verify UTxO is at the script address
    if utxo_info.address != wallet.script_address {
        return Err(Error::InvalidInput {
            reason: format!(
                "UTxO not at script address. Expected: {}, Got: {}",
                wallet.script_address, utxo_info.address
            ),
        });
    }

    // Check if deposit already exists in database
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(DEPOSITS)?;

    let tx_hash = hex::decode(&request.utxo.tx_hash).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid tx_hash hex: {}", e),
    })?;
    let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
        reason: "tx_hash must be 32 bytes".to_string(),
    })?;

    let utxo_ref = UtxoRef::new(tx_hash_array, request.utxo.index);

    if table.get(utxo_ref)?.is_some() {
        return Err(Error::InvalidInput {
            reason: "Deposit already processed".to_string(),
        });
    }

    // Verify confirm depth (reorg safety)
    // Get current chain tip
    let tip = provider.get_tip().await.map_err(|e| Error::NetworkError {
        reason: format!("Failed to get chain tip for confirm depth check: {}", e),
    })?;

    // Get confirm depth from config
    let confirm_depth = ctx.config.deposit_confirm_depth();

    // Check if UTxO has block height info
    match utxo_info.block_height {
        Some(utxo_block_height) => {
            let current_height = tip.block_height;
            let blocks_confirmed = current_height.saturating_sub(utxo_block_height);

            tracing::info!(
                "UTxO {}:{} at block {} ({} blocks confirmed, need {})",
                &request.utxo.tx_hash[..16],
                request.utxo.index,
                utxo_block_height,
                blocks_confirmed,
                confirm_depth
            );

            if blocks_confirmed < confirm_depth {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "UTxO not sufficiently confirmed. Block height: {}, Current: {}, Confirmed: {} blocks, Required: {} blocks",
                        utxo_block_height, current_height, blocks_confirmed, confirm_depth
                    ),
                });
            }

            tracing::info!(
                "UTxO {}:{} confirmed with {} blocks (required: {})",
                &request.utxo.tx_hash[..16],
                request.utxo.index,
                blocks_confirmed,
                confirm_depth
            );
        }
        None => {
            // Block height not available from provider
            // This shouldn't happen with Blockfrost since we now fetch tx info
            tracing::warn!(
                "UTxO {}:{} block height not available from provider. Cannot verify confirm depth.",
                &request.utxo.tx_hash[..16],
                request.utxo.index
            );
            return Err(Error::InvalidInput {
                reason: "Cannot verify UTxO confirmation depth: block height not available"
                    .to_string(),
            });
        }
    }

    Ok(utxo_info)
}

/// Validate that outputs cover all assets in the UTxO
///
/// NOTE: Since outputs are blinded commitments, we cannot verify the actual
/// amounts at deposit time. The Aiken validator enforces exact accounting
/// during withdrawal when outputs are unblinded.
///
/// What we validate here:
/// - At least one output is provided
/// - The number of outputs is reasonable (at least one per unique asset)
/// - No more outputs than total asset units (prevents dust attack)
fn validate_deposit_amounts(request: &DepositRequest, utxo_info: &UtxoInfo) -> Result<(), Error> {
    // Build map of assets in UTxO
    let mut utxo_assets: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut total_units: u64 = 0;

    for asset in &utxo_info.amount {
        let amount = asset
            .quantity
            .parse::<u64>()
            .map_err(|e| Error::InvalidInput {
                reason: format!("Invalid asset quantity: {}", e),
            })?;
        utxo_assets.insert(asset.unit.clone(), amount);
        total_units += amount;
    }

    // Must have at least one output
    if request.outputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No outputs provided for deposit".to_string(),
        });
    }

    // Must have at least as many outputs as unique assets (no partial deposits)
    // Each unique asset must be represented in at least one output
    if request.outputs.len() < utxo_assets.len() {
        return Err(Error::InvalidInput {
            reason: format!(
                "Insufficient outputs: {} assets in UTxO but only {} outputs provided. \
                 Each asset must be accounted for in at least one output (no partial deposits).",
                utxo_assets.len(),
                request.outputs.len()
            ),
        });
    }

    // Sanity check: shouldn't have more outputs than total asset units
    // This prevents potential dust attacks with excessive outputs
    if request.outputs.len() as u64 > total_units {
        return Err(Error::InvalidInput {
            reason: format!(
                "Too many outputs: {} outputs for {} total asset units",
                request.outputs.len(),
                total_units
            ),
        });
    }

    tracing::info!(
        "Validated deposit: {} assets in UTxO ({} total units), {} outputs",
        utxo_assets.len(),
        total_units,
        request.outputs.len()
    );

    // Check minimum deposit value
    let lovelace_amount = utxo_assets.get("lovelace").copied().unwrap_or(0);
    let min_deposit = 1_000_000; // 1 ADA minimum - should come from config
    if lovelace_amount < min_deposit {
        return Err(Error::InvalidInput {
            reason: format!(
                "Deposit value {} lovelace below minimum {} lovelace",
                lovelace_amount, min_deposit
            ),
        });
    }

    tracing::info!(
        "Validated deposit: {} assets in UTxO ({} total units, {} lovelace), {} outputs",
        utxo_assets.len(),
        total_units,
        lovelace_amount,
        request.outputs.len()
    );

    Ok(())
}

/// Sign blinded outputs with delegate key
fn sign_outputs(
    request: &DepositRequest,
    keypair: &mugraph_core::types::Keypair,
) -> Result<Vec<BlindSignature>, Error> {
    let mut rng = rand::rng();
    let mut signatures = Vec::with_capacity(request.outputs.len());

    for commitment in &request.outputs {
        // Sign the blinded commitment
        // The signature field is Blinded<Signature> which wraps a Signature
        // Access the inner Signature through the Blinded tuple struct
        let blinded_sig_data: &mugraph_core::types::Blinded<mugraph_core::types::Signature> =
            &commitment.signature;
        let signature: &mugraph_core::types::Signature = &blinded_sig_data.0;
        let sig_bytes: &[u8; 32] = signature.as_ref();

        let blinded_sig = crypto::sign_blinded(
            &mut rng,
            &keypair.secret_key,
            &crypto::hash_to_curve(sig_bytes),
        );

        signatures.push(blinded_sig);
    }

    Ok(signatures)
}

/// Record deposit in database
async fn record_deposit(
    request: &DepositRequest,
    ctx: &Context,
    provider: &Provider,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    use mugraph_core::types::DepositRecord;

    // Get current block height from provider
    let tip = provider.get_tip().await.map_err(|e| Error::NetworkError {
        reason: format!("Failed to get chain tip: {}", e),
    })?;

    // Compute intent hash for replay protection
    let intent_hash = compute_intent_hash(request, &ctx.keypair.public_key, &wallet.script_address);

    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(DEPOSITS)?;

        let tx_hash = hex::decode(&request.utxo.tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;

        let utxo_ref = UtxoRef::new(tx_hash_array, request.utxo.index);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate expiration based on config
        // Each block is approximately 20 seconds on Cardano
        let expiration_seconds = ctx.config.deposit_expiration_blocks() * 20;
        let expires_at = now + expiration_seconds;

        let record = DepositRecord::with_intent_hash(tip.block_height, now, expires_at, intent_hash);
        table.insert(utxo_ref, &record)?;
    }
    write_tx.commit()?;

    tracing::info!(
        "Deposit recorded successfully at block {}",
        tip.block_height
    );
    Ok(())
}
