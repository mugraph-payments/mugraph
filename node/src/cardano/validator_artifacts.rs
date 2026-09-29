use color_eyre::eyre::{Context, Result};

/// The compiled vault validator (the Aiken blueprint). The blueprint is
/// part of the node binary, so the node needs no Aiken sources, tools, or
/// validator files at run time. To change the validator, run
/// `aiken build` in `validator/` and build the node again.
const BLUEPRINT: &str = include_str!("../../../validator/plutus.json");

/// Returns the compiled vault validator (Plutus V3 script bytes).
pub fn validator_cbor() -> Result<Vec<u8>> {
    let blueprint: serde_json::Value = serde_json::from_str(BLUEPRINT)
        .context("Failed to parse plutus.json")?;

    let compiled_code = blueprint
        .get("validators")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .and_then(|v| v.get("compiledCode"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "Missing validator compiledCode in plutus.json"
            )
        })?;

    hex::decode(compiled_code).context("Failed to decode validator hex")
}
