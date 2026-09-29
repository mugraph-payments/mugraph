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
    addr.to_address()
        .to_bech32(None)
        .map_err(|e| format!("bech32 error: {e}"))
}

/// The vault datum: Constr(0, [user_pk_hash, node_pk_hash, intent_hash]).
fn vault_datum(
    user_pubkey_hash: &[u8; 28],
    node_pubkey_hash: &[u8; 28],
    intent_hash: &[u8; 32],
) -> csl::PlutusData {
    let mut fields = csl::PlutusList::new();
    fields.add(&csl::PlutusData::new_bytes(user_pubkey_hash.to_vec()));
    fields.add(&csl::PlutusData::new_bytes(node_pubkey_hash.to_vec()));
    fields.add(&csl::PlutusData::new_bytes(intent_hash.to_vec()));
    csl::PlutusData::new_constr_plutus_data(&csl::ConstrPlutusData::new(
        &csl::BigNum::zero(),
        &fields,
    ))
}

pub struct DepositTxParams<'a> {
    pub input_tx_hash: &'a str,
    pub input_index: u32,
    pub input_amount_lovelace: u64,
    pub deposit_amount_lovelace: u64,
    pub script_address_bech32: &'a str,
    pub user_ed25519_vk: &'a [u8; 32],
    pub node_payment_vk: &'a [u8; 28],
    pub canonical_payload: &'a [u8],
    pub change_address_bech32: &'a str,
    pub fee_lovelace: u64,
}

/// Build a deposit transaction that sends funds to a script address with an
/// inline Plutus datum containing (user_pubkey_hash, node_pubkey_hash, intent_hash).
///
/// Returns (tx_cbor, tx_hash).
pub fn build_deposit_tx(
    params: &DepositTxParams<'_>,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    let DepositTxParams {
        input_tx_hash,
        input_index,
        input_amount_lovelace,
        deposit_amount_lovelace,
        script_address_bech32,
        user_ed25519_vk,
        node_payment_vk,
        canonical_payload,
        change_address_bech32,
        fee_lovelace,
    } = params;
    // Build input
    let tx_hash_bytes = hex::decode(input_tx_hash)
        .map_err(|e| format!("bad tx hash hex: {e}"))?;
    let tx_hash = csl::TransactionHash::from_bytes(tx_hash_bytes)
        .map_err(|e| format!("bad tx hash: {e}"))?;
    let input = csl::TransactionInput::new(&tx_hash, *input_index);
    let mut inputs = csl::TransactionInputs::new();
    inputs.add(&input);

    // Build deposit output with inline datum
    let script_addr = csl::Address::from_bech32(script_address_bech32)
        .map_err(|e| format!("bad script address: {e}"))?;

    let user_pubkey_hash = blake2b_224(*user_ed25519_vk);
    let intent_hash = blake2b_256(canonical_payload);

    let datum = vault_datum(&user_pubkey_hash, node_payment_vk, &intent_hash);

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
        .checked_sub(*deposit_amount_lovelace)
        .and_then(|v| v.checked_sub(*fee_lovelace))
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

pub struct WithdrawTxParams<'a> {
    /// Script UTxO inputs to spend (tx_hash hex, index)
    pub script_inputs: &'a [(String, u32)],
    /// Total lovelace available from script inputs
    pub total_input_lovelace: u64,
    /// Destination address (bech32)
    pub destination_address: &'a str,
    /// Amount to send to destination
    pub withdraw_amount_lovelace: u64,
    /// Script address for change outputs (bech32)
    pub script_address: &'a str,
    /// Transaction fee
    pub fee_lovelace: u64,
    /// The node's payment key hash. The vault validator needs the node's
    /// signature, and the vault change datum names the node.
    pub node_pubkey_hash: &'a [u8; 28],
    /// The user key hash for the vault change datum.
    pub user_pubkey_hash: &'a [u8; 28],
}

/// Build a withdraw transaction that spends script UTxOs and sends funds
/// to a destination address, with optional change back to the script address.
///
/// Returns (tx_cbor, tx_hash).
pub fn build_withdraw_tx(
    params: &WithdrawTxParams<'_>,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    let mut inputs = csl::TransactionInputs::new();
    for (tx_hash_hex, index) in params.script_inputs {
        let tx_hash_bytes = hex::decode(tx_hash_hex)
            .map_err(|e| format!("bad input tx hash hex: {e}"))?;
        let tx_hash = csl::TransactionHash::from_bytes(tx_hash_bytes)
            .map_err(|e| format!("bad input tx hash: {e}"))?;
        inputs.add(&csl::TransactionInput::new(&tx_hash, *index));
    }

    if inputs.len() == 0 {
        return Err("no script inputs provided".to_string());
    }

    let mut outputs = csl::TransactionOutputs::new();

    // Destination output
    let dest_addr = csl::Address::from_bech32(params.destination_address)
        .map_err(|e| format!("bad destination address: {e}"))?;
    let dest_value = csl::Value::new(
        &csl::Coin::from_str(&params.withdraw_amount_lovelace.to_string())
            .map_err(|e| format!("bad withdraw amount: {e}"))?,
    );
    outputs.add(&csl::TransactionOutput::new(&dest_addr, &dest_value));

    // Change output back to script address
    let change_amount = params
        .total_input_lovelace
        .checked_sub(params.withdraw_amount_lovelace)
        .and_then(|v| v.checked_sub(params.fee_lovelace))
        .ok_or("insufficient script inputs to cover withdraw + fee")?;

    if change_amount > 0 {
        let script_addr = csl::Address::from_bech32(params.script_address)
            .map_err(|e| format!("bad script address: {e}"))?;
        let change_value = csl::Value::new(
            &csl::Coin::from_str(&change_amount.to_string())
                .map_err(|e| format!("bad change amount: {e}"))?,
        );
        // Without this datum, the vault validator can not spend the
        // change, and the funds are locked.
        let mut change =
            csl::TransactionOutput::new(&script_addr, &change_value);
        change.set_plutus_data(&vault_datum(
            params.user_pubkey_hash,
            params.node_pubkey_hash,
            &[0u8; 32],
        ));
        outputs.add(&change);
    }

    let fee = csl::Coin::from_str(&params.fee_lovelace.to_string())
        .map_err(|e| format!("bad fee: {e}"))?;

    let mut body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
    let node_key_hash =
        csl::Ed25519KeyHash::from_bytes(params.node_pubkey_hash.to_vec())
            .map_err(|e| format!("bad node key hash: {e}"))?;
    let mut required = csl::Ed25519KeyHashes::new();
    required.add(&node_key_hash);
    body.set_required_signers(&required);
    let witness_set = csl::TransactionWitnessSet::new();
    let tx = csl::Transaction::new(&body, &witness_set, None);

    let tx_cbor = tx.to_bytes();
    let tx_hash = blake2b_256(&body.to_bytes());

    Ok((tx_cbor, tx_hash))
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

        let result = build_deposit_tx(&DepositTxParams {
            input_tx_hash: &dummy_tx_hash,
            input_index: 0,
            input_amount_lovelace: 10_000_000,
            deposit_amount_lovelace: 5_000_000,
            script_address_bech32: script_addr,
            user_ed25519_vk: &vk,
            node_payment_vk: &node_pk_hash,
            canonical_payload: payload,
            change_address_bech32: &change_addr,
            fee_lovelace: 200_000,
        });

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

        let (tx_cbor, expected_hash) = build_deposit_tx(&DepositTxParams {
            input_tx_hash: &dummy_tx_hash,
            input_index: 0,
            input_amount_lovelace: 5_000_000,
            deposit_amount_lovelace: 3_000_000,
            script_address_bech32: &addr,
            user_ed25519_vk: &vk,
            node_payment_vk: &[0xCC; 28],
            canonical_payload: b"test",
            change_address_bech32: &addr,
            fee_lovelace: 200_000,
        })
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

        let (tx_cbor, tx_hash) = build_deposit_tx(&DepositTxParams {
            input_tx_hash: &dummy_tx_hash,
            input_index: 0,
            input_amount_lovelace: 5_000_000,
            deposit_amount_lovelace: 3_000_000,
            script_address_bech32: &addr,
            user_ed25519_vk: &vk,
            node_payment_vk: &[0xDD; 28],
            canonical_payload: b"test",
            change_address_bech32: &addr,
            fee_lovelace: 200_000,
        })
        .unwrap();

        let witnessed = attach_user_witness(&tx_cbor, &tx_hash, &sk).unwrap();
        assert!(!witnessed.is_empty());
        // Witnessed tx should be larger than unsigned
        assert!(witnessed.len() > tx_cbor.len());
    }

    #[test]
    fn build_withdraw_tx_produces_valid_cbor() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let dest_addr = derive_address(&vk, "preprod").unwrap();
        let script_addr = &dest_addr; // reuse for simplicity

        let (tx_cbor, tx_hash) = build_withdraw_tx(&WithdrawTxParams {
            script_inputs: &[("d".repeat(64), 0)],
            total_input_lovelace: 10_000_000,
            destination_address: &dest_addr,
            withdraw_amount_lovelace: 5_000_000,
            script_address: script_addr,
            fee_lovelace: 200_000,
            node_pubkey_hash: &[2u8; 28],
            user_pubkey_hash: &[1u8; 28],
        })
        .unwrap();

        assert!(!tx_cbor.is_empty());
        assert_ne!(tx_hash, [0u8; 32]);

        // Hash should match recomputation
        let recomputed = compute_tx_hash(&tx_cbor).unwrap();
        assert_eq!(recomputed, tx_hash);
    }

    #[test]
    fn build_withdraw_tx_no_change_when_exact() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let dest_addr = derive_address(&vk, "preprod").unwrap();

        let result = build_withdraw_tx(&WithdrawTxParams {
            script_inputs: &[("e".repeat(64), 0)],
            total_input_lovelace: 5_200_000,
            destination_address: &dest_addr,
            withdraw_amount_lovelace: 5_000_000,
            script_address: &dest_addr,
            fee_lovelace: 200_000,
            node_pubkey_hash: &[2u8; 28],
            user_pubkey_hash: &[1u8; 28],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn build_withdraw_tx_rejects_insufficient_inputs() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let dest_addr = derive_address(&vk, "preprod").unwrap();

        let result = build_withdraw_tx(&WithdrawTxParams {
            script_inputs: &[("f".repeat(64), 0)],
            total_input_lovelace: 1_000_000,
            destination_address: &dest_addr,
            withdraw_amount_lovelace: 5_000_000,
            script_address: &dest_addr,
            fee_lovelace: 200_000,
            node_pubkey_hash: &[2u8; 28],
            user_pubkey_hash: &[1u8; 28],
        });
        assert!(result.is_err());
    }

    #[test]
    fn build_withdraw_tx_rejects_empty_inputs() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let dest_addr = derive_address(&vk, "preprod").unwrap();

        let result = build_withdraw_tx(&WithdrawTxParams {
            script_inputs: &[],
            total_input_lovelace: 5_000_000,
            destination_address: &dest_addr,
            withdraw_amount_lovelace: 3_000_000,
            script_address: &dest_addr,
            fee_lovelace: 200_000,
            node_pubkey_hash: &[2u8; 28],
            user_pubkey_hash: &[1u8; 28],
        });
        assert!(result.is_err());
    }

    #[test]
    fn build_withdraw_tx_requires_the_node_and_keeps_vault_change_spendable() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let dest_addr = derive_address(&vk, "preprod").unwrap();
        let node_hash = [2u8; 28];

        let (tx_cbor, _) = build_withdraw_tx(&WithdrawTxParams {
            script_inputs: &[("a".repeat(64), 0)],
            total_input_lovelace: 10_000_000,
            destination_address: &dest_addr,
            withdraw_amount_lovelace: 5_000_000,
            script_address: &dest_addr,
            fee_lovelace: 200_000,
            node_pubkey_hash: &node_hash,
            user_pubkey_hash: &[1u8; 28],
        })
        .unwrap();

        let tx = csl::Transaction::from_bytes(tx_cbor).unwrap();
        let required: Vec<Vec<u8>> = tx
            .body()
            .required_signers()
            .expect("the node must be a required signer")
            .into_iter()
            .map(|h| h.to_bytes())
            .collect();
        assert_eq!(required, vec![node_hash.to_vec()]);

        let change = tx.body().outputs().get(1);
        let datum = change
            .plutus_data()
            .expect("vault change needs an inline datum");
        let fields = datum.as_constr_plutus_data().unwrap().data();
        assert_eq!(fields.get(1).as_bytes().unwrap(), node_hash.to_vec());
    }
}
