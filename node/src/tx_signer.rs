use color_eyre::eyre::{Context, Result};
use ed25519_dalek::{Signer, SigningKey};

/// Sign transaction body hash with the node's payment key
/// Returns the witness set bytes
pub fn sign_transaction_body(tx_body_hash: &[u8; 32], signing_key_bytes: &[u8]) -> Result<Vec<u8>> {
    // Parse the signing key
    let signing_key = SigningKey::from_bytes(
        signing_key_bytes
            .try_into()
            .map_err(|_| color_eyre::eyre::eyre!("Invalid signing key length, expected 32 bytes"))?,
    );

    // Sign the transaction hash
    let signature = signing_key.sign(tx_body_hash);

    // Return the signature bytes (64 bytes for Ed25519)
    Ok(signature.to_bytes().to_vec())
}

/// Build a transaction witness with the node signature
/// This creates a VKeyWitness structure for Cardano
pub fn build_node_witness(
    tx_hash: &[u8; 32],
    signing_key_bytes: &[u8],
    verifying_key_bytes: &[u8],
) -> Result<TransactionWitness> {
    let signature_bytes = sign_transaction_body(tx_hash, signing_key_bytes)?;

    Ok(TransactionWitness {
        vkey: verifying_key_bytes.to_vec(),
        signature: signature_bytes,
    })
}

/// Transaction witness structure
#[derive(Debug, Clone)]
pub struct TransactionWitness {
    pub vkey: Vec<u8>,      // 32 bytes Ed25519 public key
    pub signature: Vec<u8>, // 64 bytes Ed25519 signature
}

/// Attach a node witness to an existing transaction
/// The transaction should already have user witnesses attached
/// We add the node witness for script inputs
pub fn attach_witness_to_transaction(
    tx_cbor: &[u8],
    tx_hash: &[u8; 32],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<Vec<u8>> {
    // Build node witness
    let node_witness = build_node_witness(tx_hash, &wallet.payment_sk, &wallet.payment_vk)?;

    // TODO: Use whisky-csl to deserialize transaction, add witness, and reserialize
    // For now, we return the original transaction
    // In production, this would:
    // 1. Parse the transaction CBOR
    // 2. Add the node witness to the witness set
    // 3. Re-serialize to CBOR

    tracing::debug!("Attaching node witness for tx: {}", hex::encode(tx_hash));

    // Placeholder: return original CBOR
    // The actual implementation requires whisky-csl integration
    Ok(tx_cbor.to_vec())
}

/// Compute transaction body hash from CBOR
/// This is used for signing
pub fn compute_tx_hash(tx_cbor: &[u8]) -> Result<[u8; 32]> {
    // TODO: Parse transaction with whisky-csl and compute body hash
    // For now, we use blake2b-256 on the entire transaction
    // This is INCORRECT for production - we need the transaction BODY hash, not the full transaction hash
    use blake2::{Blake2b, Digest, digest::consts::U32};

    type Blake2b256 = Blake2b<U32>;
    let hash = Blake2b256::digest(tx_cbor);

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_transaction() {
        // Generate a test keypair
        let (sk, vk) = crate::cardano::generate_payment_keypair().unwrap();

        // Test message (32 bytes)
        let message = [1u8; 32];

        // Sign
        let signature = sign_transaction_body(&message, &sk).unwrap();
        assert_eq!(signature.len(), 64); // Ed25519 signature size

        // Verify
        use ed25519_dalek::{Verifier, VerifyingKey};
        let verifying_key = VerifyingKey::from_bytes(&vk.try_into().unwrap()).unwrap();
        let sig = ed25519_dalek::Signature::from_slice(&signature).unwrap();
        assert!(verifying_key.verify(&message, &sig).is_ok());
    }

    #[test]
    fn test_compute_tx_hash() {
        let tx_cbor = vec![0x00, 0x01, 0x02, 0x03];
        let hash = compute_tx_hash(&tx_cbor).unwrap();
        assert_eq!(hash.len(), 32);
    }
}
