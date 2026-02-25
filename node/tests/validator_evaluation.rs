//! UPLC evaluation integration tests for the deposit validator.
//!
//! These tests build Conway-era transactions using pallas primitives,
//! then evaluate them against the compiled Aiken validator through
//! the UPLC virtual machine. This verifies that transactions built
//! by the Rust node would actually be accepted on-chain.
//!
//! Run with: cargo test -p mugraph-node --test validator_evaluation

use blake2::{Blake2b, Digest, digest::consts::U28};
use pallas_addresses::{Network, ShelleyAddress, ShelleyDelegationPart, ShelleyPaymentPart};
use pallas_codec::{
    minicbor,
    utils::{CborWrap, MaybeIndefArray, NonEmptySet, Nullable, Set},
};
use pallas_crypto::hash::Hash;
use pallas_primitives::{
    BoundedBytes, Constr,
    conway::{
        CostModels, DatumOption, ExUnits, MintedTx, PlutusData, PlutusScript,
        PostAlonzoTransactionOutput, Redeemer, RedeemerTag, Redeemers,
        TransactionOutput, Value, WitnessSet,
    },
};
use pallas_traverse::{Era, MultiEraTx};
use uplc::{
    machine::cost_model::ExBudget,
    tx::{eval_phase_two, script_context::ResolvedInput, script_context::SlotConfig},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type Blake2b224 = Blake2b<U28>;

/// Compute blake2b-224 hash.
fn blake2b_224(data: &[u8]) -> [u8; 28] {
    let hash = Blake2b224::digest(data);
    let mut out = [0u8; 28];
    out.copy_from_slice(&hash);
    out
}

/// Compute the script hash for a PlutusV3 script.
/// PlutusV3 scripts are hashed with a 0x03 prefix tag.
fn compute_script_hash(script_cbor: &[u8]) -> Hash<28> {
    let mut hasher = Blake2b224::new();
    hasher.update(&[0x03]);
    hasher.update(script_cbor);
    let hash = hasher.finalize();
    let mut out = [0u8; 28];
    out.copy_from_slice(&hash);
    Hash::from(out)
}

/// Build a Shelley script address (enterprise, no staking) for testnet.
fn build_script_address_bytes(script_hash: &Hash<28>) -> Vec<u8> {
    let addr = ShelleyAddress::new(
        Network::Testnet,
        ShelleyPaymentPart::Script(*script_hash),
        ShelleyDelegationPart::Null,
    );
    addr.to_vec()
}

/// Build a DepositDatum as PlutusData.
/// Constructor 0, fields: [Bytes(user_hash), Bytes(node_hash), Bytes(intent_hash)]
fn build_deposit_datum(
    user_hash: &[u8],
    node_hash: &[u8],
    intent_hash: &[u8],
) -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121, // Constructor 0
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![
            PlutusData::BoundedBytes(BoundedBytes::from(user_hash.to_vec())),
            PlutusData::BoundedBytes(BoundedBytes::from(node_hash.to_vec())),
            PlutusData::BoundedBytes(BoundedBytes::from(intent_hash.to_vec())),
        ]),
    })
}

/// Build a Void redeemer (SpendRedeemer::Void = Constructor 0, no fields).
fn build_void_redeemer_data() -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121, // Constructor 0
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![]),
    })
}

/// Load the compiled validator CBOR from the Aiken build artifacts.
fn load_validator_cbor() -> Vec<u8> {
    mugraph_node::cardano::compile_validator()
        .expect("Failed to compile validator. Is `aiken` installed and on $PATH?")
}

/// Load PlutusV3 cost models from the JSON fixture.
fn load_cost_models() -> CostModels {
    let json_str = include_str!("fixtures/preprod_cost_models.json");
    let params: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let v3: Vec<i64> = params["PlutusV3"]
        .as_array()
        .expect("Fixture missing PlutusV3 cost model")
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();

    // Validate against .version file
    let version_str = include_str!("fixtures/preprod_cost_models.version");
    for line in version_str.lines() {
        if let Some(count_str) = line.strip_prefix("param_count_v3: ") {
            let expected: usize = count_str.trim().parse().unwrap();
            assert_eq!(
                v3.len(),
                expected,
                "PlutusV3 cost model has {} params but .version file says {}. \
                 Update both files together.",
                v3.len(),
                expected
            );
        }
    }

    CostModels {
        plutus_v1: None,
        plutus_v2: None,
        plutus_v3: Some(v3),
    }
}

/// Preprod slot config.
fn preprod_slot_config() -> SlotConfig {
    SlotConfig {
        zero_time: 1655683200000, // Preprod genesis time (ms)
        zero_slot: 0,
        slot_length: 1000,
    }
}

/// Build a Conway-era transaction that spends a script UTxO.
///
/// Returns (tx_cbor_bytes, resolved_inputs) ready for eval_phase_two.
fn build_spend_tx(
    script_cbor: &[u8],
    script_hash: &Hash<28>,
    datum: PlutusData,
    required_signers: Vec<Hash<28>>,
    input_tx_hash: Hash<32>,
    input_index: u64,
    input_lovelace: u64,
) -> (Vec<u8>, Vec<ResolvedInput>) {
    use pallas_primitives::conway::{PseudoTransactionBody, Tx};

    let script_address_bytes = build_script_address_bytes(script_hash);

    // The UTxO being spent (with inline datum)
    let script_utxo_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: script_address_bytes.clone().into(),
        value: Value::Coin(input_lovelace),
        datum_option: Some(DatumOption::Data(CborWrap(datum.clone()))),
        script_ref: None,
    });

    // Build a simple output (sending back to a dummy key address)
    let dummy_key_hash: [u8; 28] = [0xAA; 28];
    let change_addr = ShelleyAddress::new(
        Network::Testnet,
        ShelleyPaymentPart::Key(Hash::from(dummy_key_hash)),
        ShelleyDelegationPart::Null,
    );
    let change_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: change_addr.to_vec().into(),
        value: Value::Coin(input_lovelace.saturating_sub(2_000_000)),
        datum_option: None,
        script_ref: None,
    });

    // Transaction input
    let tx_input = pallas_primitives::TransactionInput {
        transaction_id: input_tx_hash,
        index: input_index,
    };

    // Collateral input (dummy, not validated with run_phase_one: false)
    let collateral_input = pallas_primitives::TransactionInput {
        transaction_id: Hash::from([0xBB; 32]),
        index: 0,
    };

    // Redeemer: Spend at index 0 (our script input is the only/first input)
    let redeemer = Redeemer {
        tag: RedeemerTag::Spend,
        index: 0,
        data: build_void_redeemer_data(),
        ex_units: ExUnits {
            mem: 14_000_000,
            steps: 10_000_000_000,
        },
    };

    // Required signers
    let req_signers = if required_signers.is_empty() {
        None
    } else {
        NonEmptySet::from_vec(required_signers)
    };

    // Build the transaction body
    let tx_body = PseudoTransactionBody {
        inputs: Set::from(vec![tx_input.clone()]),
        outputs: vec![change_output],
        fee: 200_000,
        ttl: Some(100_000_000),
        certificates: None,
        withdrawals: None,
        auxiliary_data_hash: None,
        validity_interval_start: None,
        mint: None,
        script_data_hash: None,
        collateral: NonEmptySet::from_vec(vec![collateral_input]),
        required_signers: req_signers,
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        voting_procedures: None,
        proposal_procedures: None,
        treasury_value: None,
        donation: None,
    };

    // Build the witness set with the PlutusV3 script and redeemer
    let witness_set = WitnessSet {
        vkeywitness: None,
        native_script: None,
        bootstrap_witness: None,
        plutus_v1_script: None,
        plutus_data: None,
        redeemer: Some(Redeemers::List(MaybeIndefArray::Def(vec![redeemer]))),
        plutus_v2_script: None,
        plutus_v3_script: NonEmptySet::from_vec(vec![PlutusScript(
            script_cbor.to_vec().into(),
        )]),
    };

    // Build the full transaction
    let tx = Tx {
        transaction_body: tx_body,
        transaction_witness_set: witness_set,
        success: true,
        auxiliary_data: Nullable::Null,
    };

    // Encode to CBOR
    let tx_bytes = minicbor::to_vec(&tx).expect("Failed to encode transaction to CBOR");

    // Build resolved inputs
    let resolved_inputs = vec![ResolvedInput {
        input: tx_input,
        output: script_utxo_output,
    }];

    (tx_bytes, resolved_inputs)
}

/// Decode CBOR bytes as a MintedTx and run eval_phase_two.
fn evaluate_tx(
    tx_bytes: &[u8],
    utxos: &[ResolvedInput],
    cost_models: &CostModels,
) -> Result<Vec<Redeemer>, uplc::tx::error::Error> {
    let mtx = MultiEraTx::decode_for_era(Era::Conway, tx_bytes)
        .expect("Failed to decode transaction CBOR");

    let tx: &MintedTx = match &mtx {
        MultiEraTx::Conway(tx) => tx,
        _ => panic!("Expected Conway-era transaction"),
    };

    let slot_config = preprod_slot_config();
    let initial_budget = ExBudget {
        cpu: 10_000_000_000,
        mem: 14_000_000,
    };

    eval_phase_two(
        tx,
        utxos,
        Some(cost_models),
        Some(&initial_budget),
        &slot_config,
        false, // run_phase_one: false (we test script logic, not ledger rules)
        |_| (),
    )
}

// ---------------------------------------------------------------------------
// Test #1: Happy path - valid user signature
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_with_valid_user_signature() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    // Deterministic test keys
    let user_key = [1u8; 32];
    let user_hash = blake2b_224(&user_key);
    let node_key = [2u8; 32];
    let node_hash = blake2b_224(&node_key);
    let intent_hash = [0u8; 32];

    let datum = build_deposit_datum(&user_hash, &node_hash, &intent_hash);

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(user_hash)], // user is required signer
        Hash::from([0x01; 32]),
        0,
        5_000_000,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed for valid user signature");

    assert_eq!(redeemers.len(), 1, "Expected exactly one redeemer result");
    let r = &redeemers[0];
    assert!(r.ex_units.steps > 0, "CPU steps should be nonzero");
    assert!(r.ex_units.mem > 0, "Memory units should be nonzero");

    println!(
        "eval_spend_with_valid_user_signature: CPU={}, Mem={}",
        r.ex_units.steps, r.ex_units.mem
    );
}

// ---------------------------------------------------------------------------
// Test #5: Failure path - missing signer
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_missing_signer() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);
    let datum = build_deposit_datum(&user_hash, &node_hash, &[0u8; 32]);

    // No required signers at all
    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![], // no signers
        Hash::from([0x02; 32]),
        0,
        5_000_000,
    );

    let result = evaluate_tx(&tx_bytes, &utxos, &cost_models);
    assert!(
        result.is_err(),
        "Expected script evaluation to fail when user signature is missing"
    );
}

// ---------------------------------------------------------------------------
// Test #6: Failure path - wrong signer
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_wrong_signer() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);
    let wrong_hash = blake2b_224(&[3u8; 32]); // different key
    let datum = build_deposit_datum(&user_hash, &node_hash, &[0u8; 32]);

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(wrong_hash)], // wrong signer
        Hash::from([0x03; 32]),
        0,
        5_000_000,
    );

    let result = evaluate_tx(&tx_bytes, &utxos, &cost_models);
    assert!(
        result.is_err(),
        "Expected script evaluation to fail when wrong signer is provided"
    );
}

// ---------------------------------------------------------------------------
// Test #9: Failure path - short hash in datum
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_short_hash_in_datum() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let short_user_hash = [0xAAu8; 20]; // 20 bytes instead of 28
    let node_hash = blake2b_224(&[2u8; 32]);
    let datum = build_deposit_datum(&short_user_hash, &node_hash, &[0u8; 32]);

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(blake2b_224(&[1u8; 32]))],
        Hash::from([0x04; 32]),
        0,
        5_000_000,
    );

    let result = evaluate_tx(&tx_bytes, &utxos, &cost_models);
    assert!(
        result.is_err(),
        "Expected script evaluation to fail with short user hash in datum"
    );
}

// ---------------------------------------------------------------------------
// Test #10: Script hash consistency
// ---------------------------------------------------------------------------

#[test]
fn eval_script_hash_matches() {
    let script_cbor = load_validator_cbor();

    // Compute the script hash the same way our node does
    let node_hash = mugraph_node::cardano::compute_script_hash(&script_cbor);

    // Compute via the canonical method (PlutusV3 tag + blake2b-224)
    let canonical_hash = compute_script_hash(&script_cbor);

    assert_eq!(
        node_hash.len(),
        28,
        "Node script hash should be 28 bytes"
    );
    assert_eq!(
        &node_hash,
        canonical_hash.as_ref(),
        "Node script hash does not match canonical PlutusV3 script hash. \
         The node may be computing the hash without the 0x03 prefix tag."
    );
}
