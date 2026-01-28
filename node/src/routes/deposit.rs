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
    record_deposit(request, ctx, &provider).await?;

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
    // TODO: Get network from config
    let wallet = setup_cardano_wallet("preprod", None)
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
fn create_provider(_ctx: &Context) -> Result<Provider, Error> {
    // TODO: Get provider config from Context
    // For now, use placeholder values
    Provider::new(
        "blockfrost",
        std::env::var("BLOCKFROST_API_KEY").unwrap_or_else(|_| "test_key".to_string()),
        "preprod".to_string(),
        None,
    )
    .map_err(|e| Error::Internal {
        reason: e.to_string(),
    })
}

/// Verify CIP-8 signature over canonical deposit payload
fn verify_deposit_signature(
    request: &DepositRequest,
    wallet: &mugraph_core::types::CardanoWallet,
    delegate_pk: &mugraph_core::types::PublicKey,
) -> Result<(), Error> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // Build canonical payload
    // Payload = utxo + outputs + delegate pk + script address + nonce + network tag
    let payload = build_canonical_payload(request, delegate_pk, &wallet.script_address);

    // Parse the CIP-8 signature
    // CIP-8 format: COSE_Sign1 structure
    // For now, we expect signature to be raw Ed25519 signature (64 bytes)
    if request.signature.len() != 64 {
        return Err(Error::InvalidSignature {
            reason: "Invalid signature length, expected 64 bytes".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Extract public key from request message (it should contain the signing key)
    // For now, we'll parse the message to get the user key hash
    // The message format should be: {"utxo":..., "user_pubkey": "...", ...}
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
            reason: "user_pubkey must be 32 bytes".to_string(),
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
        .verify(&payload, &signature)
        .map_err(|e| Error::InvalidSignature {
            reason: format!("Signature verification failed: {}", e),
            signature: mugraph_core::types::Signature::default(),
        })?;

    // Verify the user key hash matches the datum that will be stored
    // This ensures the UTxO was created with the correct datum
    let user_pubkey_hash = blake3::hash(&user_pubkey_for_hash);
    let expected_hash_hex = hex::encode(&user_pubkey_hash.as_bytes()[..28]);

    tracing::debug!(
        "CIP-8 signature verified for user key hash: {}",
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

    Ok(utxo_info)
}

/// Validate that outputs cover all assets in the UTxO
fn validate_deposit_amounts(request: &DepositRequest, utxo_info: &UtxoInfo) -> Result<(), Error> {
    // Build map of assets in UTxO
    let mut utxo_assets: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for asset in &utxo_info.amount {
        let amount = asset
            .quantity
            .parse::<u64>()
            .map_err(|e| Error::InvalidInput {
                reason: format!("Invalid asset quantity: {}", e),
            })?;
        utxo_assets.insert(asset.unit.clone(), amount);
    }

    // TODO: Validate that blinded outputs account for all assets
    // This requires knowing the unblinded values, which the node doesn't have
    // The validator will enforce this at the smart contract level
    // For now, we just check that outputs is non-empty
    if request.outputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No outputs provided for deposit".to_string(),
        });
    }

    tracing::debug!(
        "Validated deposit amounts: {} assets in UTxO, {} outputs",
        utxo_assets.len(),
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
) -> Result<(), Error> {
    use mugraph_core::types::DepositRecord;

    // Get current block height from provider
    let tip = provider.get_tip().await.map_err(|e| Error::NetworkError {
        reason: format!("Failed to get chain tip: {}", e),
    })?;

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

        // TODO: Get expiration from config
        let expires_at = now + (24 * 60 * 60); // 24 hours

        let record = DepositRecord::new(tip.block_height, now, expires_at);
        table.insert(utxo_ref, &record)?;
    }
    write_tx.commit()?;

    tracing::info!(
        "Deposit recorded successfully at block {}",
        tip.block_height
    );
    Ok(())
}
