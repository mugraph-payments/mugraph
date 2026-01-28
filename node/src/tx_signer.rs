use blake2::{Blake2b, Digest, digest::consts::U32};
use color_eyre::eyre::{Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use whisky_csl::csl;

/// Sign transaction body hash with the node's payment key
/// Returns the 64-byte Ed25519 signature
pub fn sign_transaction_body(tx_body_hash: &[u8; 32], signing_key_bytes: &[u8]) -> Result<Vec<u8>> {
    let signing_key = SigningKey::from_bytes(
        signing_key_bytes
            .try_into()
            .map_err(|_| color_eyre::eyre::eyre!("Invalid signing key length, expected 32 bytes"))?,
    );

    let signature = signing_key.sign(tx_body_hash);
    Ok(signature.to_bytes().to_vec())
}

/// Build a transaction witness with the node signature
pub fn build_node_witness(
    tx_body_hash: &[u8; 32],
    signing_key_bytes: &[u8],
    verifying_key_bytes: &[u8],
) -> Result<VKeyWitness> {
    let signature_bytes = sign_transaction_body(tx_body_hash, signing_key_bytes)?;

    Ok(VKeyWitness {
        vkey: verifying_key_bytes.try_into().map_err(|_| {
            color_eyre::eyre::eyre!("Invalid verifying key length, expected 32 bytes")
        })?,
        signature: signature_bytes
            .try_into()
            .map_err(|_| color_eyre::eyre::eyre!("Invalid signature length, expected 64 bytes"))?,
    })
}

/// VKey witness structure
#[derive(Debug, Clone)]
pub struct VKeyWitness {
    pub vkey: [u8; 32],
    pub signature: [u8; 64],
}

pub fn attach_witness_to_transaction(
    tx_cbor: &[u8],
    tx_body_hash: &[u8; 32],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<Vec<u8>> {
    tracing::debug!(
        "Attaching node witness for tx: {}",
        hex::encode(tx_body_hash)
    );

    // Parse transaction
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).context("Invalid transaction CBOR")?;

    // Build witness using CSL helpers
    let priv_key = csl::PrivateKey::from_normal_bytes(&wallet.payment_sk)
        .context("Invalid payment signing key")?;
    let tx_hash =
        csl::TransactionHash::from_bytes(tx_body_hash.to_vec()).context("Invalid tx hash bytes")?;
    let vkey_witness = csl::make_vkey_witness(&tx_hash, &priv_key);

    // Merge witness into witness set
    let mut witness_set = tx.witness_set();
    let mut vkeys = witness_set.vkeys().unwrap_or_else(csl::Vkeywitnesses::new);
    vkeys.add(&vkey_witness);
    witness_set.set_vkeys(&vkeys);

    // Rebuild transaction preserving is_valid and auxiliary data
    let body = tx.body();
    let aux = tx.auxiliary_data();
    let is_valid = tx.is_valid();
    let mut new_tx = csl::Transaction::new(&body, &witness_set, aux);
    new_tx.set_is_valid(is_valid);

    let bytes = new_tx.to_bytes();
    tracing::info!("Node witness attached successfully");
    Ok(bytes)
}

/// Compute transaction body hash from CBOR
pub fn compute_tx_hash(tx_cbor: &[u8]) -> Result<[u8; 32]> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec())
        .context("Invalid transaction CBOR for hashing")?;
    let body_bytes = tx.body().to_bytes();

    type Blake2b256 = Blake2b<U32>;
    let hash = Blake2b256::digest(&body_bytes);

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_transaction() {
        let (sk, vk) = crate::cardano::generate_payment_keypair().unwrap();
        let message = [1u8; 32];

        let signature = sign_transaction_body(&message, &sk).unwrap();
        assert_eq!(signature.len(), 64);

        use ed25519_dalek::{Signature as EdSignature, Verifier, VerifyingKey};
        let verifying_key = VerifyingKey::from_bytes(&vk.try_into().unwrap()).unwrap();
        let sig = EdSignature::from_slice(&signature).unwrap();
        assert!(verifying_key.verify(&message, &sig).is_ok());
    }

    #[test]
    fn test_compute_tx_hash() {
        let tx_cbor = vec![0x82, 0xa0, 0xa0];
        let hash = compute_tx_hash(&tx_cbor).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_build_node_witness() {
        let (sk, vk) = crate::cardano::generate_payment_keypair().unwrap();
        let tx_hash = [1u8; 32];

        let witness = build_node_witness(&tx_hash, &sk, &vk).unwrap();
        assert_eq!(witness.vkey.len(), 32);
        assert_eq!(witness.signature.len(), 64);
    }

    #[test]
    fn test_attach_witness() {
        let (sk, vk) = crate::cardano::generate_payment_keypair().unwrap();

        // Create a simple transaction: [empty_map, empty_map]
        let tx_cbor = vec![0x82, 0xa0, 0xa0];
        let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

        let wallet = mugraph_core::types::CardanoWallet::new(
            sk,
            vk,
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let result = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet);
        assert!(result.is_ok());

        let new_tx = result.unwrap();
        // Should be longer than original (witness added)
        assert!(new_tx.len() > tx_cbor.len());
    }
}
