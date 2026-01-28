use std::{path::Path, process::Command};

use color_eyre::eyre::{Context, Result};
use mugraph_core::types::CardanoWallet;
use rand::TryRngCore;

/// Generate a new Ed25519 keypair for Cardano payments
pub fn generate_payment_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let mut sk = vec![0u8; 32];
    rand::rng()
        .try_fill_bytes(&mut sk)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to generate random bytes: {}", e))?;

    // For Ed25519, we derive the public key from the secret key
    // Using ed25519-dalek for key derivation
    use ed25519_dalek::SigningKey;

    // Clone sk before converting since try_into consumes the vec
    let sk_array: [u8; 32] = sk
        .clone()
        .try_into()
        .map_err(|_| color_eyre::eyre::eyre!("Failed to convert secret key bytes"))?;
    let signing_key = SigningKey::from_bytes(&sk_array);
    let vk = signing_key.verifying_key().to_bytes().to_vec();

    Ok((sk, vk))
}

/// Import a payment signing key from hex string
pub fn import_payment_key(hex_sk: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let sk =
        hex::decode(hex_sk.trim_start_matches("0x")).context("Failed to decode hex signing key")?;

    if sk.len() != 32 {
        return Err(color_eyre::eyre::eyre!(
            "Signing key must be 32 bytes, got {}",
            sk.len()
        ));
    }

    // Derive public key
    use ed25519_dalek::SigningKey;
    let signing_key = SigningKey::from_bytes(
        &sk.clone()
            .try_into()
            .map_err(|_| color_eyre::eyre::eyre!("Failed to convert secret key bytes"))?,
    );
    let vk = signing_key.verifying_key().to_bytes().to_vec();

    Ok((sk, vk))
}

/// Path to the Aiken validator artifacts
pub fn get_validator_dir() -> Result<std::path::PathBuf> {
    let validator_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get parent directory"))?
        .join("validator");

    if !validator_dir.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Validator directory not found at {:?}",
            validator_dir
        ));
    }

    Ok(validator_dir)
}

/// Check if validator artifacts exist and are up to date
pub fn validator_artifacts_exist() -> Result<bool> {
    let validator_dir = get_validator_dir()?;
    let plutus_json_path = validator_dir.join("build").join("plutus.json");

    if !plutus_json_path.exists() {
        return Ok(false);
    }

    // Check if source files are newer than the artifact
    let _build_dir = validator_dir.join("build");
    let plutus_modified = std::fs::metadata(&plutus_json_path)
        .and_then(|m| m.modified())
        .ok();

    // Check validators directory for .ak files
    let validators_dir = validator_dir.join("validators");
    if validators_dir.exists() {
        for entry in std::fs::read_dir(validators_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "ak")
                && let Ok(source_modified) = std::fs::metadata(&path).and_then(|m| m.modified())
                && let Some(plutus_time) = plutus_modified
                && source_modified > plutus_time
            {
                // Source is newer than artifact, needs rebuild
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Load validator CBOR from existing artifacts
pub fn load_validator_cbor() -> Result<Vec<u8>> {
    let validator_dir = get_validator_dir()?;
    let plutus_json_path = validator_dir.join("build").join("plutus.json");

    let plutus_content =
        std::fs::read_to_string(&plutus_json_path).context("Failed to read plutus.json")?;

    let plutus_json: serde_json::Value =
        serde_json::from_str(&plutus_content).context("Failed to parse plutus.json")?;

    // Extract the script CBOR from the first validator
    let validators = plutus_json
        .get("validators")
        .and_then(|v| v.as_array())
        .ok_or_else(|| color_eyre::eyre::eyre!("No validators found in plutus.json"))?;

    if validators.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "Empty validators array in plutus.json"
        ));
    }

    let compiled_code = validators[0]
        .get("compiledCode")
        .and_then(|c| c.as_str())
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing compiledCode in validator"))?;

    // compiledCode is hex-encoded CBOR
    let cbor = hex::decode(compiled_code).context("Failed to decode CBOR hex")?;

    Ok(cbor)
}

/// Compile Aiken validator and return the CBOR bytes
/// Looks for validator in the validator/ directory relative to project root
/// Only compiles if artifacts don't exist or source is newer
pub fn compile_validator() -> Result<Vec<u8>> {
    // Check if artifacts exist and are up to date
    if validator_artifacts_exist()? {
        tracing::info!("Using cached validator artifacts");
        return load_validator_cbor();
    }

    tracing::info!("Validator artifacts not found or outdated, compiling...");

    let validator_dir = get_validator_dir()?;

    // Run aiken build
    let output = Command::new("aiken")
        .arg("build")
        .current_dir(&validator_dir)
        .output()
        .context("Failed to run 'aiken build'. Is aiken installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(color_eyre::eyre::eyre!("Aiken build failed: {}", stderr));
    }

    tracing::info!("Validator compiled successfully");

    // Load the freshly compiled validator
    load_validator_cbor()
}

/// Compute script hash from CBOR (Blake2b-224)
pub fn compute_script_hash(cbor: &[u8]) -> Vec<u8> {
    use blake2::{Blake2b, Digest, digest::consts::U28};

    type Blake2b224 = Blake2b<U28>;
    let hash = Blake2b224::digest(cbor);
    hash.to_vec()
}

/// Build script address from hash and network
/// Uses Shelley address format directly instead of bech32
pub fn build_script_address(script_hash: &[u8], network: &str) -> Result<String> {
    // Determine network tag and header byte for script address
    // Header byte: 0xF0 | network_tag
    // Network tags: Mainnet = 1, Testnet/Preprod/Preview = 0
    let (hrp, network_tag) = match network {
        "mainnet" => ("addr", 1u8), // Mainnet script address
        "preprod" | "preview" | "testnet" => ("addr_test", 0u8), // Testnet script address
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Unknown network: {}. Use mainnet, preprod, preview, or testnet",
                network
            ));
        }
    };

    // Construct address bytes using Shelley binary format
    // Header byte: 0xF0 (script address type) | network_tag
    let header: u8 = 0xF0 | network_tag;

    // Address = header (1 byte) + payment part (28 bytes script hash)
    let mut address_bytes = vec![header];
    address_bytes.extend_from_slice(script_hash);

    // Encode as bech32 using bech32 crate v0.11 API
    let hrp = bech32::Hrp::parse(hrp).map_err(|e| color_eyre::eyre::eyre!("Invalid HRP: {}", e))?;
    let address = bech32::encode::<bech32::Bech32>(hrp, &address_bytes)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to encode bech32: {}", e))?;

    Ok(address)
}

/// Create or load Cardano wallet
pub async fn setup_cardano_wallet(network: &str, payment_sk: Option<&str>) -> Result<CardanoWallet> {
    // Generate or import payment keypair
    let (sk, vk) = if let Some(hex_sk) = payment_sk {
        import_payment_key(hex_sk)?
    } else {
        generate_payment_keypair()?
    };

    // Compile validator
    let cbor = compile_validator()?;

    // Compute script hash
    let script_hash = compute_script_hash(&cbor);

    // Build script address
    let script_address = build_script_address(&script_hash, network)?;

    Ok(CardanoWallet::new(
        sk,
        vk,
        cbor,
        script_hash,
        script_address,
        network.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        assert_eq!(sk.len(), 32);
        assert_eq!(vk.len(), 32);
    }

    #[test]
    fn test_import_key() {
        // Generate a key first
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = hex::encode(&sk);

        // Import it back
        let (imported_sk, imported_vk) = import_payment_key(&hex_sk).unwrap();
        assert_eq!(sk, imported_sk);
        assert_eq!(vk, imported_vk);
    }

    #[test]
    fn test_script_hash() {
        let cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_script_hash(&cbor);
        assert_eq!(hash.len(), 28); // Blake2b-224
    }
}
