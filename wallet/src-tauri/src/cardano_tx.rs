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

/// The protocol parameters that a withdrawal needs for its fee and its
/// script data hash.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolParams {
    pub min_fee_a: u64,
    pub min_fee_b: u64,
    pub price_mem: f64,
    pub price_step: f64,
    pub plutus_v3_cost_model: Vec<i64>,
}

/// A pure-ADA UTxO of the user that the transaction puts up as
/// collateral. The ledger takes the collateral only if a script fails.
pub struct CollateralInput {
    pub tx_hash: String,
    pub index: u32,
    pub lovelace: u64,
    /// Where the unused part of the collateral goes.
    pub return_address: String,
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
    /// The node's payment key hash. The vault validator needs the node's
    /// signature, and the vault change datum names the node.
    pub node_pubkey_hash: &'a [u8; 28],
    /// The user key hash for the vault change datum.
    pub user_pubkey_hash: &'a [u8; 28],
    /// The vault validator (Plutus V3), as in the blueprint.
    pub script_cbor: &'a [u8],
    pub collateral: &'a CollateralInput,
    pub protocol: &'a ProtocolParams,
}

pub struct WithdrawTx {
    pub tx_cbor: Vec<u8>,
    pub tx_hash: [u8; 32],
    pub fee: u64,
}

/// The execution units for each vault input. The validator uses about
/// 28,000 memory units and 9.3 million steps (see the Plutus evaluation
/// tests), so this leaves a margin of more than three times.
const SPEND_MEM: u64 = 100_000;
const SPEND_STEPS: u64 = 40_000_000;

/// The size of a vkey witness in CBOR, for the fee estimate. The wallet
/// adds one witness for the collateral, and the node adds one.
const VKEY_WITNESS_SIZE: u64 = 101;
const ADDED_WITNESSES: u64 = 2;

/// The ledger requires collateral of 150% of the fee.
const COLLATERAL_PERCENT: u64 = 150;

/// The smallest vault change output. The ledger rejects outputs below its
/// minimum UTxO value (about 1 ADA for an output with a datum), so a
/// smaller change goes to the fee.
const MIN_VAULT_CHANGE: u64 = 1_000_000;

/// Build a withdraw transaction that spends vault UTxOs with the vault
/// validator, and sends funds to a destination address, with change back
/// to the vault.
///
/// The transaction carries the validator, one `Void` redeemer for each
/// vault input, the script data hash, and collateral. It needs a witness
/// for the collateral (the wallet) and for the node.
pub fn build_withdraw_tx(
    params: &WithdrawTxParams<'_>,
) -> Result<WithdrawTx, String> {
    if params.script_inputs.is_empty() {
        return Err("no script inputs provided".to_string());
    }

    // Two rounds: the fee changes the change output, and thus the size.
    let mut fee = 0u64;
    for _ in 0..2 {
        let tx = assemble_withdraw_tx(params, fee)?;
        fee = withdraw_fee(&tx, params.protocol);
    }

    // Change below the minimum UTxO value goes to the fee.
    let change = params
        .total_input_lovelace
        .checked_sub(params.withdraw_amount_lovelace)
        .and_then(|v| v.checked_sub(fee))
        .ok_or("insufficient script inputs to cover withdraw + fee")?;
    if change > 0 && change < MIN_VAULT_CHANGE {
        fee += change;
    }

    let tx = assemble_withdraw_tx(params, fee)?;
    let tx_cbor = tx.to_bytes();
    let tx_hash = blake2b_256(&tx.body().to_bytes());

    Ok(WithdrawTx {
        tx_cbor,
        tx_hash,
        fee,
    })
}

fn withdraw_fee(tx: &csl::Transaction, protocol: &ProtocolParams) -> u64 {
    let size = tx.to_bytes().len() as u64 + ADDED_WITNESSES * VKEY_WITNESS_SIZE;
    let redeemers =
        tx.witness_set().redeemers().map(|r| r.len()).unwrap_or(0) as u64;
    let script_fee = (redeemers as f64)
        * (SPEND_MEM as f64 * protocol.price_mem
            + SPEND_STEPS as f64 * protocol.price_step);

    // A small margin, because the fee field itself can grow the size.
    protocol.min_fee_a * size
        + protocol.min_fee_b
        + script_fee.ceil() as u64
        + 1_000
}

fn coin(amount: u64) -> csl::Coin {
    csl::BigNum::from(amount)
}

fn assemble_withdraw_tx(
    params: &WithdrawTxParams<'_>,
    fee: u64,
) -> Result<csl::Transaction, String> {
    // The ledger sorts inputs by transaction hash, then index. A spend
    // redeemer points to its input by that order.
    let mut sorted: Vec<(Vec<u8>, u32)> = params
        .script_inputs
        .iter()
        .map(|(hash, index)| {
            hex::decode(hash)
                .map(|bytes| (bytes, *index))
                .map_err(|e| format!("bad input tx hash hex: {e}"))
        })
        .collect::<Result<_, _>>()?;
    sorted.sort();

    let mut inputs = csl::TransactionInputs::new();
    let mut redeemers = csl::Redeemers::new();
    for (i, (hash, index)) in sorted.iter().enumerate() {
        let tx_hash = csl::TransactionHash::from_bytes(hash.clone())
            .map_err(|e| format!("bad input tx hash: {e}"))?;
        inputs.add(&csl::TransactionInput::new(&tx_hash, *index));
        redeemers.add(&csl::Redeemer::new(
            &csl::RedeemerTag::new_spend(),
            &csl::BigNum::from(i as u64),
            &csl::PlutusData::new_empty_constr_plutus_data(&csl::BigNum::zero()),
            &csl::ExUnits::new(
                &csl::BigNum::from(SPEND_MEM),
                &csl::BigNum::from(SPEND_STEPS),
            ),
        ));
    }

    let mut outputs = csl::TransactionOutputs::new();

    // Destination output
    let dest_addr = csl::Address::from_bech32(params.destination_address)
        .map_err(|e| format!("bad destination address: {e}"))?;
    outputs.add(&csl::TransactionOutput::new(
        &dest_addr,
        &csl::Value::new(&coin(params.withdraw_amount_lovelace)),
    ));

    // Change output back to script address
    let change_amount = params
        .total_input_lovelace
        .checked_sub(params.withdraw_amount_lovelace)
        .and_then(|v| v.checked_sub(fee))
        .ok_or("insufficient script inputs to cover withdraw + fee")?;

    if change_amount > 0 {
        let script_addr = csl::Address::from_bech32(params.script_address)
            .map_err(|e| format!("bad script address: {e}"))?;
        // Without this datum, the vault validator can not spend the
        // change, and the funds are locked.
        let mut change = csl::TransactionOutput::new(
            &script_addr,
            &csl::Value::new(&coin(change_amount)),
        );
        change.set_plutus_data(&vault_datum(
            params.user_pubkey_hash,
            params.node_pubkey_hash,
            &[0u8; 32],
        ));
        outputs.add(&change);
    }

    let mut body =
        csl::TransactionBody::new_tx_body(&inputs, &outputs, &coin(fee));

    let node_key_hash =
        csl::Ed25519KeyHash::from_bytes(params.node_pubkey_hash.to_vec())
            .map_err(|e| format!("bad node key hash: {e}"))?;
    let mut required = csl::Ed25519KeyHashes::new();
    required.add(&node_key_hash);
    body.set_required_signers(&required);

    // Collateral, with the unused part going back to the user.
    let collateral = params.collateral;
    let collateral_hash =
        csl::TransactionHash::from_hex(&collateral.tx_hash)
            .map_err(|e| format!("bad collateral tx hash: {e}"))?;
    let mut collateral_inputs = csl::TransactionInputs::new();
    collateral_inputs.add(&csl::TransactionInput::new(
        &collateral_hash,
        collateral.index,
    ));
    body.set_collateral(&collateral_inputs);

    let total_collateral = (fee * COLLATERAL_PERCENT).div_ceil(100);
    let collateral_return =
        collateral
            .lovelace
            .checked_sub(total_collateral)
            .ok_or("collateral UTxO is too small for the fee")?;
    let return_addr = csl::Address::from_bech32(&collateral.return_address)
        .map_err(|e| format!("bad collateral return address: {e}"))?;
    body.set_collateral_return(&csl::TransactionOutput::new(
        &return_addr,
        &csl::Value::new(&coin(collateral_return)),
    ));
    body.set_total_collateral(&coin(total_collateral));

    // The script data hash binds the redeemers and the cost model.
    let mut cost_models = csl::Costmdls::new();
    cost_models.insert(
        &csl::Language::new_plutus_v3(),
        &csl::CostModel::from(
            params
                .protocol
                .plutus_v3_cost_model
                .iter()
                .map(|&c| c as i128)
                .collect::<Vec<i128>>(),
        ),
    );
    body.set_script_data_hash(&csl::hash_script_data(
        &redeemers,
        &cost_models,
        None,
    ));

    let mut scripts = csl::PlutusScripts::new();
    scripts.add(&csl::PlutusScript::new_v3(params.script_cbor.to_vec()));

    let mut witness_set = csl::TransactionWitnessSet::new();
    witness_set.set_plutus_scripts(&scripts);
    witness_set.set_redeemers(&redeemers);

    Ok(csl::Transaction::new(&body, &witness_set, None))
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

    fn test_protocol() -> ProtocolParams {
        ProtocolParams {
            min_fee_a: 44,
            min_fee_b: 155_381,
            price_mem: 0.0577,
            price_step: 0.0000721,
            plutus_v3_cost_model: vec![0; 297],
        }
    }

    fn withdraw_params<'a>(
        script_inputs: &'a [(String, u32)],
        total: u64,
        amount: u64,
        address: &'a str,
        collateral: &'a CollateralInput,
        protocol: &'a ProtocolParams,
    ) -> WithdrawTxParams<'a> {
        WithdrawTxParams {
            script_inputs,
            total_input_lovelace: total,
            destination_address: address,
            withdraw_amount_lovelace: amount,
            script_address: address,
            node_pubkey_hash: &[2u8; 28],
            user_pubkey_hash: &[1u8; 28],
            script_cbor: &[0x46, 0x01, 0x00, 0x00, 0x22, 0x49, 0x9d],
            collateral,
            protocol,
        }
    }

    fn test_collateral(address: &str) -> CollateralInput {
        CollateralInput {
            tx_hash: "c".repeat(64),
            index: 0,
            lovelace: 5_000_000,
            return_address: address.to_string(),
        }
    }

    #[test]
    fn build_withdraw_tx_produces_valid_cbor() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();
        let inputs = [("d".repeat(64), 0)];

        let built = build_withdraw_tx(&withdraw_params(
            &inputs,
            10_000_000,
            5_000_000,
            &addr,
            &collateral,
            &protocol,
        ))
        .unwrap();

        assert!(!built.tx_cbor.is_empty());
        assert_eq!(compute_tx_hash(&built.tx_cbor).unwrap(), built.tx_hash);
    }

    #[test]
    fn build_withdraw_tx_no_change_output_when_exact() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();
        let inputs = [("e".repeat(64), 0)];

        // Learn the fee, then spend exactly payout + fee.
        let fee = build_withdraw_tx(&withdraw_params(
            &inputs,
            10_000_000,
            5_000_000,
            &addr,
            &collateral,
            &protocol,
        ))
        .unwrap()
        .fee;
        let built = build_withdraw_tx(&withdraw_params(
            &inputs,
            5_000_000 + fee,
            5_000_000,
            &addr,
            &collateral,
            &protocol,
        ))
        .unwrap();

        // Without a change output the transaction is smaller, so a little
        // change is left. It is below the minimum UTxO value, so it goes
        // to the fee.
        let tx = csl::Transaction::from_bytes(built.tx_cbor).unwrap();
        assert_eq!(tx.body().outputs().len(), 1);
        assert_eq!(built.fee, fee);
    }

    #[test]
    fn build_withdraw_tx_never_makes_dust_change() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();
        let inputs = [("e".repeat(64), 0)];

        for extra in [1u64, 500_000, 999_999] {
            let fee = build_withdraw_tx(&withdraw_params(
                &inputs,
                10_000_000,
                5_000_000,
                &addr,
                &collateral,
                &protocol,
            ))
            .unwrap()
            .fee;
            let built = build_withdraw_tx(&withdraw_params(
                &inputs,
                5_000_000 + fee + extra,
                5_000_000,
                &addr,
                &collateral,
                &protocol,
            ))
            .unwrap();

            let tx = csl::Transaction::from_bytes(built.tx_cbor).unwrap();
            let outputs = tx.body().outputs();
            for i in 0..outputs.len() {
                let lovelace: u64 =
                    outputs.get(i).amount().coin().to_str().parse().unwrap();
                assert!(lovelace >= MIN_VAULT_CHANGE, "dust output {lovelace}");
            }
        }
    }

    #[test]
    fn build_withdraw_tx_rejects_insufficient_inputs() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();
        let inputs = [("f".repeat(64), 0)];

        assert!(
            build_withdraw_tx(&withdraw_params(
                &inputs,
                1_000_000,
                5_000_000,
                &addr,
                &collateral,
                &protocol,
            ))
            .is_err()
        );
    }

    #[test]
    fn build_withdraw_tx_rejects_empty_inputs() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();

        assert!(
            build_withdraw_tx(&withdraw_params(
                &[],
                5_000_000,
                3_000_000,
                &addr,
                &collateral,
                &protocol,
            ))
            .is_err()
        );
    }

    #[test]
    fn build_withdraw_tx_requires_the_node_and_keeps_vault_change_spendable() {
        let sk = test_ed25519_key();
        let vk = sk.verifying_key().to_bytes();
        let addr = derive_address(&vk, "preprod").unwrap();
        let collateral = test_collateral(&addr);
        let protocol = test_protocol();
        let inputs = [("a".repeat(64), 0)];

        let built = build_withdraw_tx(&withdraw_params(
            &inputs,
            10_000_000,
            5_000_000,
            &addr,
            &collateral,
            &protocol,
        ))
        .unwrap();

        let tx = csl::Transaction::from_bytes(built.tx_cbor).unwrap();
        let required: Vec<Vec<u8>> = tx
            .body()
            .required_signers()
            .expect("the node must be a required signer")
            .into_iter()
            .map(|h| h.to_bytes())
            .collect();
        assert_eq!(required, vec![vec![2u8; 28]]);

        let change = tx.body().outputs().get(1);
        let datum = change
            .plutus_data()
            .expect("vault change needs an inline datum");
        let fields = datum.as_constr_plutus_data().unwrap().data();
        assert_eq!(fields.get(1).as_bytes().unwrap(), vec![2u8; 28]);
    }
}
