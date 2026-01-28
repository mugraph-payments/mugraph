use color_eyre::eyre::{Context, Result};
use ed25519_dalek::{Signer, SigningKey};

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

/// Attach a node witness to an existing transaction
///
/// This implementation uses minicbor to manipulate the transaction CBOR directly,
/// avoiding the dependency on cardano-serialization-lib which has version conflicts
/// with wasm-bindgen in the current dependency tree.
pub fn attach_witness_to_transaction(
    tx_cbor: &[u8],
    tx_body_hash: &[u8; 32],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<Vec<u8>> {
    // Build node witness
    let node_witness = build_node_witness(tx_body_hash, &wallet.payment_sk, &wallet.payment_vk)?;

    tracing::debug!(
        "Attaching node witness for tx: {}",
        hex::encode(tx_body_hash)
    );

    // Parse transaction and add witness
    let tx_with_witness = add_vkey_witness_to_transaction(tx_cbor, &node_witness)?;

    tracing::info!("Node witness attached successfully");
    Ok(tx_with_witness)
}

/// Add a vkey witness to a transaction
///
/// Transaction structure in CBOR:
/// [
///   body: { ... },           // Map - transaction body
///   witness_set: {           // Map - witness set
///     0: [[vkey, sig], ...], // Optional: vkey witnesses (key 0)
///     ...                    // Other witness types
///   },
///   is_valid: bool,          // Boolean - phase-2 validation result
///   auxiliary_data: any      // Optional auxiliary data
/// ]
fn add_vkey_witness_to_transaction(tx_cbor: &[u8], witness: &VKeyWitness) -> Result<Vec<u8>> {
    use minicbor::{Decoder, Encoder};

    let mut decoder = Decoder::new(tx_cbor);

    // Read transaction array
    let len = decoder
        .array()
        .context("Failed to decode transaction as array")?
        .ok_or_else(|| color_eyre::eyre::eyre!("Indefinite length transaction not supported"))?;

    if len < 2 {
        return Err(color_eyre::eyre::eyre!(
            "Transaction must have at least body and witness_set, got {} elements",
            len
        ));
    }

    // Extract body
    let body_pos = decoder.position();
    decoder.skip().context("Failed to skip transaction body")?;
    let body_end = decoder.position();
    let body = &tx_cbor[body_pos..body_end];

    // Extract and modify witness set
    let witness_pos = decoder.position();
    decoder.skip().context("Failed to skip witness set")?;
    let witness_end = decoder.position();
    let witness_set_cbor = &tx_cbor[witness_pos..witness_end];

    // Add witness to witness set
    let new_witness_set = add_witness_to_set(witness_set_cbor, witness)?;

    // Extract remaining elements (is_valid and optional auxiliary_data)
    let mut remaining = Vec::new();
    for _ in 2..len {
        let start = decoder.position();
        decoder
            .skip()
            .context("Failed to skip transaction element")?;
        let end = decoder.position();
        remaining.push(&tx_cbor[start..end]);
    }

    // Reconstruct transaction
    let mut result = Vec::new();
    let mut encoder = Encoder::new(&mut result);

    // Encode array header
    encoder
        .array(len as u64)
        .context("Failed to encode array header")?;

    // Encode body (copy as-is)
    result.extend_from_slice(body);

    // Encode new witness set
    result.extend_from_slice(&new_witness_set);

    // Encode remaining elements
    for elem in remaining {
        result.extend_from_slice(elem);
    }

    Ok(result)
}

/// Add a witness to an existing witness set
///
/// Witness set structure:
/// { 0: [vkey_witnesses] } where vkey_witnesses is [[vkey: bytes, sig: bytes], ...]
fn add_witness_to_set(witness_set_cbor: &[u8], witness: &VKeyWitness) -> Result<Vec<u8>> {
    use minicbor::{Decoder, Encoder};

    let mut decoder = Decoder::new(witness_set_cbor);

    // Parse existing witness set map
    let map_len = decoder
        .map()
        .context("Failed to decode witness set as map")?;

    let mut entries: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut vkey_witnesses: Vec<VKeyWitness> = Vec::new();

    // Parse existing entries
    let iterations = match map_len {
        Some(n) => n as usize,
        None => {
            // Indefinite length - parse until break
            usize::MAX
        }
    };

    for _ in 0..iterations {
        match decoder.u64() {
            Ok(key) => {
                if key == 0 {
                    // Parse existing vkey witnesses
                    vkey_witnesses = parse_vkey_witnesses_from_decoder(&mut decoder)?;
                } else {
                    // Store other entries as raw CBOR
                    let start = decoder.position();
                    decoder.skip().context("Failed to skip witness set entry")?;
                    let end = decoder.position();
                    entries.push((key, witness_set_cbor[start..end].to_vec()));
                }
            }
            Err(_) => break, // End of map
        }
    }

    // Add new witness
    vkey_witnesses.push(witness.clone());

    // Encode new witness set
    let mut result = Vec::new();
    let mut encoder = Encoder::new(&mut result);

    // Map with vkey witnesses + other entries
    encoder
        .map((1 + entries.len()) as u64)
        .context("Failed to encode witness set map header")?;

    // Encode vkey witnesses (key 0)
    encoder.u64(0).context("Failed to encode vkey key")?;
    encoder
        .array(vkey_witnesses.len() as u64)
        .context("Failed to encode vkey array header")?;

    for vw in &vkey_witnesses {
        encoder
            .array(2)
            .context("Failed to encode witness pair header")?;
        encoder.bytes(&vw.vkey).context("Failed to encode vkey")?;
        encoder
            .bytes(&vw.signature)
            .context("Failed to encode signature")?;
    }

    // Encode other entries - finish encoder first
    drop(encoder);

    for (key, value) in entries {
        // Encode key
        minicbor::encode(key, &mut result).context("Failed to encode witness key")?;
        // Value is already CBOR encoded, append directly
        result.extend_from_slice(&value);
    }

    Ok(result)
}

/// Parse vkey witnesses from decoder
fn parse_vkey_witnesses_from_decoder(decoder: &mut minicbor::Decoder) -> Result<Vec<VKeyWitness>> {
    let arr_len = decoder
        .array()
        .context("Failed to decode vkey witnesses array")?;
    let arr_len = arr_len
        .ok_or_else(|| color_eyre::eyre::eyre!("Indefinite length vkey array not supported"))?;

    let mut witnesses = Vec::new();

    for _ in 0..arr_len {
        let pair_len = decoder.array().context("Failed to decode witness pair")?;
        let pair_len = pair_len
            .ok_or_else(|| color_eyre::eyre::eyre!("Indefinite length witness pair not supported"))?;

        if pair_len != 2 {
            return Err(color_eyre::eyre::eyre!(
                "VKey witness pair must have 2 elements, got {}",
                pair_len
            ));
        }

        let vkey: Vec<u8> = decoder.bytes().context("Failed to decode vkey")?.to_vec();
        let signature: Vec<u8> = decoder
            .bytes()
            .context("Failed to decode signature")?
            .to_vec();

        if vkey.len() != 32 {
            return Err(color_eyre::eyre::eyre!(
                "Invalid vkey length: expected 32, got {}",
                vkey.len()
            ));
        }
        if signature.len() != 64 {
            return Err(color_eyre::eyre::eyre!(
                "Invalid signature length: expected 64, got {}",
                signature.len()
            ));
        }

        witnesses.push(VKeyWitness {
            vkey: vkey.try_into().unwrap(),
            signature: signature.try_into().unwrap(),
        });
    }

    Ok(witnesses)
}

/// Compute transaction body hash from CBOR
pub fn compute_tx_hash(tx_cbor: &[u8]) -> Result<[u8; 32]> {
    use blake2::{Blake2b, Digest, digest::consts::U32};

    let mut decoder = minicbor::Decoder::new(tx_cbor);

    let tx_len = decoder.array().context("Expected transaction array")?;
    let tx_len = tx_len
        .ok_or_else(|| color_eyre::eyre::eyre!("Indefinite transaction length not supported"))?;

    if tx_len < 1 {
        return Err(color_eyre::eyre::eyre!("Transaction must have a body"));
    }

    let body_start = decoder.position();
    decoder.skip().context("Failed to skip transaction body")?;
    let body_end = decoder.position();
    let body_bytes = &tx_cbor[body_start..body_end];

    type Blake2b256 = Blake2b<U32>;
    let hash = Blake2b256::digest(body_bytes);

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
