use whisky_csl::csl;

use crate::cip8::{blake2b_224, blake2b_256};

/// Derive a Cardano address from a payment verification key for a given network.
pub fn derive_address(
    payment_vk: &[u8; 32],
    network: &str,
) -> Result<String, String> {
    let pub_key = csl::PublicKey::from_bytes(payment_vk)
        .map_err(|e| format!("invalid payment vk: {e}"))?;

    let network_id = match network {
        "mainnet" => 1u8,
        _ => 0u8, // preprod, preview, testnet
    };

    let cred = csl::Credential::from_keyhash(&pub_key.hash());
    let addr = csl::EnterpriseAddress::new(network_id, &cred);
    Ok(addr
        .to_address()
        .to_bech32(None)
        .map_err(|e| format!("bech32 error: {e}"))?)
}

/// Build a deposit transaction that sends funds to a script address with an
/// inline Plutus datum containing (user_pubkey_hash, node_pubkey_hash, intent_hash).
///
/// Returns (tx_cbor, tx_hash) as hex strings.
pub fn build_deposit_tx(
    input_tx_hash: &str,
    input_index: u32,
    input_amount_lovelace: u64,
    deposit_amount_lovelace: u64,
    script_address_bech32: &str,
    user_ed25519_vk: &[u8; 32],
    node_payment_vk: &[u8; 28],
    canonical_payload: &[u8],
    change_address_bech32: &str,
    fee_lovelace: u64,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    // Build input
    let tx_hash_bytes = hex::decode(input_tx_hash)
        .map_err(|e| format!("bad tx hash hex: {e}"))?;
    let tx_hash = csl::TransactionHash::from_bytes(tx_hash_bytes)
        .map_err(|e| format!("bad tx hash: {e}"))?;
    let input = csl::TransactionInput::new(&tx_hash, input_index);
    let mut inputs = csl::TransactionInputs::new();
    inputs.add(&input);

    // Build deposit output with inline datum
    let script_addr = csl::Address::from_bech32(script_address_bech32)
        .map_err(|e| format!("bad script address: {e}"))?;

    let user_pubkey_hash = blake2b_224(user_ed25519_vk);
    let intent_hash = blake2b_256(canonical_payload);

    // Build Plutus datum: Constr(0, [user_pk_hash, node_pk_hash, intent_hash])
    let mut datum_fields = csl::PlutusList::new();
    datum_fields.add(&csl::PlutusData::new_bytes(user_pubkey_hash.to_vec()));
    datum_fields.add(&csl::PlutusData::new_bytes(node_payment_vk.to_vec()));
    datum_fields.add(&csl::PlutusData::new_bytes(intent_hash.to_vec()));
    let datum = csl::PlutusData::new_constr_plutus_data(
        &csl::ConstrPlutusData::new(&csl::BigNum::zero(), &datum_fields),
    );

    let deposit_value = csl::Value::new(
        &csl::Coin::from_str(&deposit_amount_lovelace.to_string())
            .map_err(|e| format!("bad deposit amount: {e}"))?,
    );

    let mut deposit_output =
        csl::TransactionOutput::new(&script_addr, &deposit_value);
    deposit_output.set_plutus_data(&datum);

    let mut outputs = csl::TransactionOutputs::new();
    outputs.add(&deposit_output);

    // Change output
    let change_amount = input_amount_lovelace
        .checked_sub(deposit_amount_lovelace)
        .and_then(|v| v.checked_sub(fee_lovelace))
        .ok_or("insufficient input to cover deposit + fee")?;

    if change_amount > 0 {
        let change_addr = csl::Address::from_bech32(change_address_bech32)
            .map_err(|e| format!("bad change address: {e}"))?;
        let change_value = csl::Value::new(
            &csl::Coin::from_str(&change_amount.to_string())
                .map_err(|e| format!("bad change amount: {e}"))?,
        );
        let change_output =
            csl::TransactionOutput::new(&change_addr, &change_value);
        outputs.add(&change_output);
    }

    let fee = csl::Coin::from_str(&fee_lovelace.to_string())
        .map_err(|e| format!("bad fee: {e}"))?;

    let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);

    let witness_set = csl::TransactionWitnessSet::new();
    let tx = csl::Transaction::new(&body, &witness_set, None);

    let tx_cbor = tx.to_bytes();

    // Compute tx hash from body bytes (Blake2b-256)
    let body_bytes = body.to_bytes();
    let tx_hash_result = blake2b_256(&body_bytes);

    Ok((tx_cbor, tx_hash_result))
}

/// Attach a user witness (Ed25519 signature) to a transaction.
pub fn attach_user_witness(
    tx_cbor: &[u8],
    tx_body_hash: &[u8; 32],
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<Vec<u8>, String> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec())
        .map_err(|e| format!("bad tx cbor: {e}"))?;

    let priv_key_bytes = signing_key.to_bytes();
    let priv_key = csl::PrivateKey::from_normal_bytes(&priv_key_bytes)
        .map_err(|e| format!("bad private key: {e}"))?;

    let csl_tx_hash = csl::TransactionHash::from_bytes(tx_body_hash.to_vec())
        .map_err(|e| format!("bad tx hash: {e}"))?;
    let vkey_witness = csl::make_vkey_witness(&csl_tx_hash, &priv_key);

    let mut witness_set = tx.witness_set();
    let mut vkeys = witness_set.vkeys().unwrap_or_else(csl::Vkeywitnesses::new);
    vkeys.add(&vkey_witness);
    witness_set.set_vkeys(&vkeys);

    let body = tx.body();
    let aux = tx.auxiliary_data();
    let is_valid = tx.is_valid();
    let mut new_tx = csl::Transaction::new(&body, &witness_set, aux);
    new_tx.set_is_valid(is_valid);

    Ok(new_tx.to_bytes())
}

/// Compute the Blake2b-256 hash of a transaction's body from CBOR.
pub fn compute_tx_hash(tx_cbor: &[u8]) -> Result<[u8; 32], String> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec())
        .map_err(|e| format!("bad tx cbor: {e}"))?;
    let body_bytes = tx.body().to_bytes();
    Ok(blake2b_256(&body_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ed25519_key() -> ed25519_dalek::SigningKey {
        let mut bytes = [0u8; 32];
        rand::Fill::fill(&mut bytes, &mut rand::rng());
        ed25519_dalek::SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn derive_address_testnet() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        assert!(addr.starts_with("addr_test1"));
    }

    #[test]
    fn derive_address_mainnet() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "mainnet").unwrap();
        assert!(addr.starts_with("addr1"));
    }

    #[test]
    fn build_deposit_tx_produces_valid_cbor() {
        // Use a testnet address from the CSL library
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let change_addr = derive_address(&vk, "preprod").unwrap();

        // A minimal script address (we just need something parseable)
        let script_addr = &change_addr; // reuse for simplicity

        let dummy_tx_hash = "a".repeat(64);
        let node_pk_hash = [0xBBu8; 28];
        let payload = b"canonical payload";

        let result = build_deposit_tx(
            &dummy_tx_hash,
            0,
            10_000_000,
            5_000_000,
            script_addr,
            &vk,
            &node_pk_hash,
            payload,
            &change_addr,
            200_000,
        );

        assert!(
            result.is_ok(),
            "build_deposit_tx failed: {:?}",
            result.err()
        );
        let (tx_cbor, tx_hash) = result.unwrap();
        assert!(!tx_cbor.is_empty());
        assert_ne!(tx_hash, [0u8; 32]);
    }

    #[test]
    fn compute_tx_hash_matches_build() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let dummy_tx_hash = "b".repeat(64);

        let (tx_cbor, expected_hash) = build_deposit_tx(
            &dummy_tx_hash,
            0,
            5_000_000,
            3_000_000,
            &addr,
            &vk,
            &[0xCC; 28],
            b"test",
            &addr,
            200_000,
        )
        .unwrap();

        let computed = compute_tx_hash(&tx_cbor).unwrap();
        assert_eq!(computed, expected_hash);
    }

    #[test]
    fn attach_user_witness_adds_signature() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let dummy_tx_hash = "c".repeat(64);

        let (tx_cbor, tx_hash) = build_deposit_tx(
            &dummy_tx_hash,
            0,
            5_000_000,
            3_000_000,
            &addr,
            &vk,
            &[0xDD; 28],
            b"test",
            &addr,
            200_000,
        )
        .unwrap();

        let witnessed = attach_user_witness(&tx_cbor, &tx_hash, &sk).unwrap();
        assert!(!witnessed.is_empty());
        // Witnessed tx should be larger than unsigned
        assert!(witnessed.len() > tx_cbor.len());
    }
}
