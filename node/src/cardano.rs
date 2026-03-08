mod address;
mod keys;
mod validator_artifacts;
mod wallet;

pub use address::{build_script_address, compute_script_hash};
pub use keys::{generate_payment_keypair, import_payment_key};

use std::{path::Path, process::Command};

use color_eyre::eyre::{Context, Result};
use mugraph_core::types::CardanoWallet;

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
    let plutus_json_path = validator_dir.join("plutus.json");

    if !plutus_json_path.exists() {
        return Ok(false);
    }

    // Check if source files are newer than the artifact
    let plutus_modified = std::fs::metadata(&plutus_json_path)
        .and_then(|m| m.modified())
        .ok();

    // Check if any .ak source file or aiken.toml is newer than plutus.json.
    // Scans both validators/ and lib/ recursively.
    let dirs_to_check = [
        validator_dir.join("validators"),
        validator_dir.join("lib"),
    ];

    fn any_ak_newer_than(
        dir: &Path,
        plutus_time: Option<std::time::SystemTime>,
    ) -> Result<bool> {
        if !dir.exists() {
            return Ok(false);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if any_ak_newer_than(&path, plutus_time)? {
                    return Ok(true);
                }
            } else if path.extension().is_some_and(|ext| ext == "ak")
                && let Ok(source_modified) = std::fs::metadata(&path).and_then(|m| m.modified())
                    && let Some(plutus_time) = plutus_time
                        && source_modified > plutus_time {
                            return Ok(true);
                        }
        }
        Ok(false)
    }

    for dir in &dirs_to_check {
        if any_ak_newer_than(dir, plutus_modified)? {
            return Ok(false);
        }
    }

    // Also check aiken.toml (dependency/compiler changes)
    let aiken_toml = validator_dir.join("aiken.toml");
    if aiken_toml.exists()
        && let Ok(toml_modified) = std::fs::metadata(&aiken_toml).and_then(|m| m.modified())
            && let Some(plutus_time) = plutus_modified
                && toml_modified > plutus_time {
                    return Ok(false);
                }

    Ok(true)
}

/// Load validator CBOR from existing artifacts
pub fn load_validator_cbor() -> Result<Vec<u8>> {
    let validator_dir = get_validator_dir()?;
    let plutus_json_path = validator_dir.join("plutus.json");

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
    fn import_payment_key_accepts_0x_prefix() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = format!("0x{}", hex::encode(&sk));

        let (imported_sk, imported_vk) = import_payment_key(&hex_sk).unwrap();
        assert_eq!(sk, imported_sk);
        assert_eq!(vk, imported_vk);
    }

    #[test]
    fn import_payment_key_rejects_wrong_length() {
        let err = import_payment_key(&hex::encode([7u8; 31])).unwrap_err();
        assert!(err.to_string().contains("Signing key must be 32 bytes"));
    }

    #[test]
    fn test_script_hash() {
        let cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_script_hash(&cbor);
        assert_eq!(hash.len(), 28); // Blake2b-224
    }

    #[test]
    fn compute_script_hash_matches_known_vector() {
        let cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_script_hash(&cbor);
        assert_eq!(hex::encode(hash), "7c4412a4936b244f2f1c645bf039c49d57b8cd18108b1a9ae5220a42");
    }

    #[test]
    fn build_script_address_rejects_unknown_network() {
        let err = build_script_address(&[0u8; 28], "staging").unwrap_err();
        assert!(err.to_string().contains("Unknown network: staging"));
    }

    #[tokio::test]
    async fn setup_cardano_wallet_preserves_imported_key_and_network() {
        let (sk, vk) = generate_payment_keypair().unwrap();
        let hex_sk = hex::encode(&sk);

        let wallet = setup_cardano_wallet("preprod", Some(&hex_sk))
            .await
            .expect("wallet from imported key");

        assert_eq!(wallet.payment_sk, sk);
        assert_eq!(wallet.payment_vk, vk);
        assert_eq!(wallet.network, "preprod");
        assert_eq!(wallet.script_hash.len(), 28);
        assert!(wallet.script_address.starts_with("addr_test1"));
    }
}
