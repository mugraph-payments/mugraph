use std::{path::Path, process::Command};

use color_eyre::eyre::{Context, Result};

/// Path to the Aiken validator artifacts
pub fn get_validator_dir() -> Result<std::path::PathBuf> {
    let validator_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| {
            color_eyre::eyre::eyre!("Failed to get parent directory")
        })?
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

    let plutus_modified = std::fs::metadata(&plutus_json_path)
        .and_then(|m| m.modified())
        .ok();

    let dirs_to_check =
        [validator_dir.join("validators"), validator_dir.join("lib")];

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
                && let Ok(source_modified) =
                    std::fs::metadata(&path).and_then(|m| m.modified())
                && let Some(plutus_time) = plutus_time
                && source_modified > plutus_time
            {
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

    let aiken_toml = validator_dir.join("aiken.toml");
    if aiken_toml.exists()
        && let Ok(toml_modified) =
            std::fs::metadata(&aiken_toml).and_then(|m| m.modified())
        && let Some(plutus_time) = plutus_modified
        && toml_modified > plutus_time
    {
        return Ok(false);
    }

    Ok(true)
}

/// Load validator CBOR from existing artifacts
pub fn load_validator_cbor() -> Result<Vec<u8>> {
    let validator_dir = get_validator_dir()?;
    let plutus_json_path = validator_dir.join("plutus.json");

    let plutus_content = std::fs::read_to_string(&plutus_json_path)
        .context("Failed to read plutus.json")?;

    let plutus_json: serde_json::Value = serde_json::from_str(&plutus_content)
        .context("Failed to parse plutus.json")?;

    let validators = plutus_json
        .get("validators")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
        color_eyre::eyre::eyre!("No validators found in plutus.json")
    })?;

    if validators.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "Empty validators array in plutus.json"
        ));
    }

    let compiled_code = validators[0]
        .get("compiledCode")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            color_eyre::eyre::eyre!("Missing compiledCode in validator")
        })?;

    let cbor =
        hex::decode(compiled_code).context("Failed to decode CBOR hex")?;

    Ok(cbor)
}

/// Compile Aiken validator and return the CBOR bytes
pub fn compile_validator() -> Result<Vec<u8>> {
    if validator_artifacts_exist()? {
        tracing::info!("Using cached validator artifacts");
        return load_validator_cbor();
    }

    tracing::info!("Validator artifacts not found or outdated, compiling...");

    let validator_dir = get_validator_dir()?;

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
    load_validator_cbor()
}
