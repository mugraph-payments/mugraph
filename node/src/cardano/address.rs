use color_eyre::eyre::Result;

/// Compute script hash from CBOR (Blake2b-224).
/// Per Cardano ledger rules, the hash input is: language_tag || script_cbor
/// where language_tag is 0x03 for PlutusV3.
pub fn compute_script_hash(cbor: &[u8]) -> Vec<u8> {
    use blake2::{Blake2b, Digest, digest::consts::U28};

    type Blake2b224 = Blake2b<U28>;
    let mut hasher = Blake2b224::new();
    hasher.update([0x03]); // PlutusV3 tag
    hasher.update(cbor);
    hasher.finalize().to_vec()
}

/// Build script address from hash and network
/// Uses Shelley address format directly instead of bech32
pub fn build_script_address(script_hash: &[u8], network: &str) -> Result<String> {
    let (hrp, network_tag) = match network {
        "mainnet" => ("addr", 1u8),
        "preprod" | "preview" | "testnet" => ("addr_test", 0u8),
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Unknown network: {}. Use mainnet, preprod, preview, or testnet",
                network
            ));
        }
    };

    let header: u8 = 0xF0 | network_tag;
    let mut address_bytes = vec![header];
    address_bytes.extend_from_slice(script_hash);

    let hrp = bech32::Hrp::parse(hrp).map_err(|e| color_eyre::eyre::eyre!("Invalid HRP: {}", e))?;
    let address = bech32::encode::<bech32::Bech32>(hrp, &address_bytes)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to encode bech32: {}", e))?;

    Ok(address)
}
