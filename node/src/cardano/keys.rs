use color_eyre::eyre::{Context, Result};
use rand::TryRngCore;

/// Generate a new Ed25519 keypair for Cardano payments
pub fn generate_payment_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let mut sk = vec![0u8; 32];
    rand::rng().try_fill_bytes(&mut sk).map_err(|e| {
        color_eyre::eyre::eyre!("Failed to generate random bytes: {}", e)
    })?;

    use ed25519_dalek::SigningKey;

    let sk_array: [u8; 32] = sk.clone().try_into().map_err(|_| {
        color_eyre::eyre::eyre!("Failed to convert secret key bytes")
    })?;
    let signing_key = SigningKey::from_bytes(&sk_array);
    let vk = signing_key.verifying_key().to_bytes().to_vec();

    Ok((sk, vk))
}

/// Import a payment signing key from hex string
pub fn import_payment_key(hex_sk: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    let sk = hex::decode(hex_sk.trim_start_matches("0x"))
        .context("Failed to decode hex signing key")?;

    if sk.len() != 32 {
        return Err(color_eyre::eyre::eyre!(
            "Signing key must be 32 bytes, got {}",
            sk.len()
        ));
    }

    use ed25519_dalek::SigningKey;
    let signing_key =
        SigningKey::from_bytes(&sk.clone().try_into().map_err(|_| {
            color_eyre::eyre::eyre!("Failed to convert secret key bytes")
        })?);
    let vk = signing_key.verifying_key().to_bytes().to_vec();

    Ok((sk, vk))
}
