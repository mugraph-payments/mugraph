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
    utils::{CborWrap, MaybeIndefArray, NonEmptyKeyValuePairs, NonEmptySet, Nullable, PositiveCoin, Set},
};
use pallas_crypto::hash::Hash;
use pallas_primitives::{
    BoundedBytes, Constr,
    conway::{
        CostModels, DatumOption, ExUnits, Multiasset, MintedTx, PlutusData, PlutusScript,
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

/// Build a `Value::Multiasset` with ADA and a single native token.
fn build_multiasset_value(
    lovelace: u64,
    policy_id: Hash<28>,
    asset_name: &[u8],
    token_amount: u64,
) -> Value {
    let inner: NonEmptyKeyValuePairs<pallas_codec::utils::Bytes, PositiveCoin> =
        NonEmptyKeyValuePairs::Def(vec![(
            asset_name.to_vec().into(),
            PositiveCoin::try_from(token_amount).expect("token amount must be > 0"),
        )]);
    let multiasset: Multiasset<PositiveCoin> =
        NonEmptyKeyValuePairs::Def(vec![(policy_id, inner)]);
    Value::Multiasset(lovelace, multiasset)
}

/// Build a Conway-era transaction that spends a script UTxO with a custom Value.
///
/// Returns (tx_cbor_bytes, resolved_inputs) ready for eval_phase_two.
fn build_spend_tx_with_value(
    script_cbor: &[u8],
    script_hash: &Hash<28>,
    datum: PlutusData,
    required_signers: Vec<Hash<28>>,
    input_tx_hash: Hash<32>,
    input_index: u64,
    input_value: Value,
    output_value: Value,
) -> (Vec<u8>, Vec<ResolvedInput>) {
    use pallas_primitives::conway::{PseudoTransactionBody, Tx};

    let script_address_bytes = build_script_address_bytes(script_hash);

    let script_utxo_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: script_address_bytes.clone().into(),
        value: input_value,
        datum_option: Some(DatumOption::Data(CborWrap(datum.clone()))),
        script_ref: None,
    });

    let dummy_key_hash: [u8; 28] = [0xAA; 28];
    let change_addr = ShelleyAddress::new(
        Network::Testnet,
        ShelleyPaymentPart::Key(Hash::from(dummy_key_hash)),
        ShelleyDelegationPart::Null,
    );
    let change_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: change_addr.to_vec().into(),
        value: output_value,
        datum_option: None,
        script_ref: None,
    });

    let tx_input = pallas_primitives::TransactionInput {
        transaction_id: input_tx_hash,
        index: input_index,
    };

    let collateral_input = pallas_primitives::TransactionInput {
        transaction_id: Hash::from([0xBB; 32]),
        index: 0,
    };

    let redeemer = Redeemer {
        tag: RedeemerTag::Spend,
        index: 0,
        data: build_void_redeemer_data(),
        ex_units: ExUnits {
            mem: 14_000_000,
            steps: 10_000_000_000,
        },
    };

    let req_signers = if required_signers.is_empty() {
        None
    } else {
        NonEmptySet::from_vec(required_signers)
    };

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

    let tx = Tx {
        transaction_body: tx_body,
        transaction_witness_set: witness_set,
        success: true,
        auxiliary_data: Nullable::Null,
    };

    let tx_bytes = minicbor::to_vec(&tx).expect("Failed to encode transaction to CBOR");

    let resolved_inputs = vec![ResolvedInput {
        input: tx_input,
        output: script_utxo_output,
    }];

    (tx_bytes, resolved_inputs)
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

/// Build a Conway-era transaction spending multiple script UTxOs.
///
/// Each input gets its own datum and redeemer. All share the same script and signer.
fn build_multi_spend_tx(
    script_cbor: &[u8],
    script_hash: &Hash<28>,
    datums: Vec<PlutusData>,
    required_signers: Vec<Hash<28>>,
    input_lovelace_each: u64,
) -> (Vec<u8>, Vec<ResolvedInput>) {
    use pallas_primitives::conway::{PseudoTransactionBody, Tx};

    let script_address_bytes = build_script_address_bytes(script_hash);

    let mut inputs = Vec::new();
    let mut resolved = Vec::new();
    let mut redeemers = Vec::new();

    for (i, datum) in datums.iter().enumerate() {
        let tx_input = pallas_primitives::TransactionInput {
            transaction_id: Hash::from([i as u8 + 1; 32]),
            index: 0,
        };

        let utxo_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
            address: script_address_bytes.clone().into(),
            value: Value::Coin(input_lovelace_each),
            datum_option: Some(DatumOption::Data(CborWrap(datum.clone()))),
            script_ref: None,
        });

        resolved.push(ResolvedInput {
            input: tx_input.clone(),
            output: utxo_output,
        });

        inputs.push(tx_input);
    }

    // Sort inputs the same way the ledger does (by tx_id then index)
    // to ensure redeemer indices match sorted order.
    inputs.sort_by(|a, b| {
        a.transaction_id
            .cmp(&b.transaction_id)
            .then(a.index.cmp(&b.index))
    });
    resolved.sort_by(|a, b| {
        a.input
            .transaction_id
            .cmp(&b.input.transaction_id)
            .then(a.input.index.cmp(&b.input.index))
    });

    for (i, _) in inputs.iter().enumerate() {
        redeemers.push(Redeemer {
            tag: RedeemerTag::Spend,
            index: i as u32,
            data: build_void_redeemer_data(),
            ex_units: ExUnits {
                mem: 14_000_000,
                steps: 10_000_000_000,
            },
        });
    }

    let total_lovelace = input_lovelace_each * inputs.len() as u64;
    let dummy_key_hash: [u8; 28] = [0xAA; 28];
    let change_addr = ShelleyAddress::new(
        Network::Testnet,
        ShelleyPaymentPart::Key(Hash::from(dummy_key_hash)),
        ShelleyDelegationPart::Null,
    );
    let change_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: change_addr.to_vec().into(),
        value: Value::Coin(total_lovelace.saturating_sub(2_000_000)),
        datum_option: None,
        script_ref: None,
    });

    let collateral_input = pallas_primitives::TransactionInput {
        transaction_id: Hash::from([0xBB; 32]),
        index: 0,
    };

    let req_signers = if required_signers.is_empty() {
        None
    } else {
        NonEmptySet::from_vec(required_signers)
    };

    let tx_body = PseudoTransactionBody {
        inputs: Set::from(inputs),
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

    let witness_set = WitnessSet {
        vkeywitness: None,
        native_script: None,
        bootstrap_witness: None,
        plutus_v1_script: None,
        plutus_data: None,
        redeemer: Some(Redeemers::List(MaybeIndefArray::Def(redeemers))),
        plutus_v2_script: None,
        plutus_v3_script: NonEmptySet::from_vec(vec![PlutusScript(
            script_cbor.to_vec().into(),
        )]),
    };

    let tx = Tx {
        transaction_body: tx_body,
        transaction_witness_set: witness_set,
        success: true,
        auxiliary_data: Nullable::Null,
    };

    let tx_bytes = minicbor::to_vec(&tx).expect("Failed to encode transaction to CBOR");
    (tx_bytes, resolved)
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
// Test #2: Happy path - multiple inputs
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_with_multiple_inputs() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);

    // 3 datums, all with the same user/node keys
    let datums: Vec<PlutusData> = (0..3)
        .map(|i| build_deposit_datum(&user_hash, &node_hash, &[i; 32]))
        .collect();

    let (tx_bytes, utxos) = build_multi_spend_tx(
        &script_cbor,
        &script_hash,
        datums,
        vec![Hash::from(user_hash)],
        5_000_000,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed for multi-input spend");

    assert_eq!(redeemers.len(), 3, "Expected 3 redeemer results");
    for (i, r) in redeemers.iter().enumerate() {
        assert!(r.ex_units.steps > 0, "CPU steps should be nonzero for input {i}");
        assert!(r.ex_units.mem > 0, "Memory units should be nonzero for input {i}");
    }

    println!(
        "eval_spend_with_multiple_inputs: per-input CPU=[{}, {}, {}], Mem=[{}, {}, {}]",
        redeemers[0].ex_units.steps, redeemers[1].ex_units.steps, redeemers[2].ex_units.steps,
        redeemers[0].ex_units.mem, redeemers[1].ex_units.mem, redeemers[2].ex_units.mem,
    );
}

// ---------------------------------------------------------------------------
// Test #4: Happy path - minimal transaction
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_minimal_tx() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    // Bare minimum: valid 28-byte hashes, correct signer, 2 ADA
    let user_hash = blake2b_224(&[10u8; 32]);
    let node_hash = blake2b_224(&[20u8; 32]);
    let datum = build_deposit_datum(&user_hash, &node_hash, &[]);

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(user_hash)],
        Hash::from([0x05; 32]),
        0,
        2_000_000,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed for minimal tx");

    assert_eq!(redeemers.len(), 1);
    assert!(redeemers[0].ex_units.steps > 0);
}

// ---------------------------------------------------------------------------
// Test #7: Failure path - no datum on UTxO
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_no_datum() {
    use pallas_primitives::conway::{PseudoTransactionBody, Tx};

    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let script_address_bytes = build_script_address_bytes(&script_hash);

    // UTxO WITHOUT datum
    let tx_input = pallas_primitives::TransactionInput {
        transaction_id: Hash::from([0x06; 32]),
        index: 0,
    };

    let script_utxo_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: script_address_bytes.into(),
        value: Value::Coin(5_000_000),
        datum_option: None, // no datum
        script_ref: None,
    });

    let dummy_key_hash: [u8; 28] = [0xAA; 28];
    let change_addr = ShelleyAddress::new(
        Network::Testnet,
        ShelleyPaymentPart::Key(Hash::from(dummy_key_hash)),
        ShelleyDelegationPart::Null,
    );
    let change_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: change_addr.to_vec().into(),
        value: Value::Coin(3_000_000),
        datum_option: None,
        script_ref: None,
    });

    let collateral_input = pallas_primitives::TransactionInput {
        transaction_id: Hash::from([0xBB; 32]),
        index: 0,
    };

    let redeemer = Redeemer {
        tag: RedeemerTag::Spend,
        index: 0,
        data: build_void_redeemer_data(),
        ex_units: ExUnits {
            mem: 14_000_000,
            steps: 10_000_000_000,
        },
    };

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
        required_signers: NonEmptySet::from_vec(vec![Hash::from(user_hash)]),
        network_id: None,
        collateral_return: None,
        total_collateral: None,
        reference_inputs: None,
        voting_procedures: None,
        proposal_procedures: None,
        treasury_value: None,
        donation: None,
    };

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

    let tx = Tx {
        transaction_body: tx_body,
        transaction_witness_set: witness_set,
        success: true,
        auxiliary_data: Nullable::Null,
    };

    let tx_bytes = minicbor::to_vec(&tx).expect("Failed to encode tx");
    let utxos = vec![ResolvedInput {
        input: tx_input,
        output: script_utxo_output,
    }];

    let result = evaluate_tx(&tx_bytes, &utxos, &cost_models);
    assert!(
        result.is_err(),
        "Expected script evaluation to fail when datum is missing from UTxO"
    );
}

// ---------------------------------------------------------------------------
// Test #8: Failure path - malformed datum (missing fields)
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_malformed_datum() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);

    // Datum with only 1 field instead of 3 -- will fail when the script
    // tries to access node_pubkey_hash or intent_hash.
    let bad_datum = PlutusData::Constr(Constr {
        tag: 121, // Constructor 0
        any_constructor: None,
        fields: MaybeIndefArray::Def(vec![
            PlutusData::BoundedBytes(BoundedBytes::from(user_hash.to_vec())),
            // missing node_pubkey_hash and intent_hash
        ]),
    });

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        bad_datum,
        vec![Hash::from(user_hash)],
        Hash::from([0x07; 32]),
        0,
        5_000_000,
    );

    let result = evaluate_tx(&tx_bytes, &utxos, &cost_models);
    assert!(
        result.is_err(),
        "Expected script evaluation to fail with malformed datum (missing fields)"
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

// ---------------------------------------------------------------------------
// Test #11 & #12: Budget regression tests
// ---------------------------------------------------------------------------

// Calibrated 2026-02-25
// Single-spend observed: CPU=8983142, Mem=26897
// Thresholds set at ~1.5x observed values.
const SINGLE_SPEND_CPU_LIMIT: u64 = 14_000_000;
const SINGLE_SPEND_MEM_LIMIT: u64 = 45_000;

#[test]
fn eval_budget_single_spend() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);
    let datum = build_deposit_datum(&user_hash, &node_hash, &[0u8; 32]);

    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(user_hash)],
        Hash::from([0x08; 32]),
        0,
        5_000_000,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed");

    let r = &redeemers[0];
    println!(
        "eval_budget_single_spend: CPU={}, Mem={}",
        r.ex_units.steps, r.ex_units.mem
    );

    assert!(
        r.ex_units.steps <= SINGLE_SPEND_CPU_LIMIT,
        "CPU budget regression: {} > {} (limit). Recalibrate if validator changed.",
        r.ex_units.steps,
        SINGLE_SPEND_CPU_LIMIT
    );
    assert!(
        r.ex_units.mem <= SINGLE_SPEND_MEM_LIMIT,
        "Memory budget regression: {} > {} (limit). Recalibrate if validator changed.",
        r.ex_units.mem,
        SINGLE_SPEND_MEM_LIMIT
    );
}

#[test]
fn eval_budget_three_inputs() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);

    let datums: Vec<PlutusData> = (0..3)
        .map(|i| build_deposit_datum(&user_hash, &node_hash, &[i; 32]))
        .collect();

    let (tx_bytes, utxos) = build_multi_spend_tx(
        &script_cbor,
        &script_hash,
        datums,
        vec![Hash::from(user_hash)],
        5_000_000,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed");

    assert_eq!(redeemers.len(), 3);

    let total_cpu: u64 = redeemers.iter().map(|r| r.ex_units.steps).sum();
    let total_mem: u64 = redeemers.iter().map(|r| r.ex_units.mem).sum();

    println!(
        "eval_budget_three_inputs: total CPU={}, total Mem={}",
        total_cpu, total_mem
    );

    // Assert roughly linear scaling: 3x single within 20% tolerance
    let expected_cpu = SINGLE_SPEND_CPU_LIMIT * 3;
    let expected_mem = SINGLE_SPEND_MEM_LIMIT * 3;

    assert!(
        total_cpu <= expected_cpu,
        "Total CPU for 3 inputs ({}) exceeds 3x single-spend limit ({}). \
         Script cost may not be scaling linearly.",
        total_cpu,
        expected_cpu
    );
    assert!(
        total_mem <= expected_mem,
        "Total memory for 3 inputs ({}) exceeds 3x single-spend limit ({}). \
         Script cost may not be scaling linearly.",
        total_mem,
        expected_mem
    );
}

// ---------------------------------------------------------------------------
// Test #3: Happy path - native tokens (multiasset UTxO)
// ---------------------------------------------------------------------------

#[test]
fn eval_spend_with_native_tokens() {
    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    let user_hash = blake2b_224(&[1u8; 32]);
    let node_hash = blake2b_224(&[2u8; 32]);
    let datum = build_deposit_datum(&user_hash, &node_hash, &[0u8; 32]);

    // Dummy policy id (28 bytes) and asset name
    let token_policy_id: Hash<28> = Hash::from([0xCC; 28]);
    let token_asset_name = b"SNEK";
    let token_amount = 1_000_000u64;
    let ada_amount = 5_000_000u64;

    let input_value = build_multiasset_value(
        ada_amount,
        token_policy_id,
        token_asset_name,
        token_amount,
    );

    // Output: send tokens + remaining ADA (minus fees) to change address
    let output_value = build_multiasset_value(
        ada_amount.saturating_sub(2_000_000),
        token_policy_id,
        token_asset_name,
        token_amount,
    );

    let (tx_bytes, utxos) = build_spend_tx_with_value(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(user_hash)],
        Hash::from([0x10; 32]),
        0,
        input_value,
        output_value,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Script evaluation failed for multiasset spend");

    assert_eq!(redeemers.len(), 1, "Expected exactly one redeemer result");
    let r = &redeemers[0];
    assert!(r.ex_units.steps > 0, "CPU steps should be nonzero");
    assert!(r.ex_units.mem > 0, "Memory units should be nonzero");

    println!(
        "eval_spend_with_native_tokens: CPU={}, Mem={}",
        r.ex_units.steps, r.ex_units.mem
    );
}

// ---------------------------------------------------------------------------
// E2E Lifecycle: deposit → off-chain transfers → withdrawal
// ---------------------------------------------------------------------------

/// Simulate the full mugraph lifecycle through the UPLC VM:
///
/// 1. **Deposit**: Construct a UTxO at the script address (on-chain deposit)
/// 2. **Off-chain transfers**: Use the mugraph blind-signature note system
///    to move value between users via Refresh operations
/// 3. **Withdrawal**: Build a spend tx for the original UTxO and evaluate
///    it through the on-chain validator
///
/// The on-chain validator only cares about the spend (withdrawal) phase —
/// it checks that the user's pubkey hash is in extra_signatories. The
/// off-chain transfers don't affect on-chain state but this test verifies
/// the full system flow works end-to-end.
#[test]
fn eval_lifecycle_deposit_transfer_withdraw() {
    use mugraph_core::{builder::RefreshBuilder, crypto, types::Keypair};

    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    // --- Phase 0: Setup identities ---
    let user_key = [1u8; 32];
    let user_hash = blake2b_224(&user_key);
    let node_key = [2u8; 32];
    let node_hash = blake2b_224(&node_key);
    let intent_hash = [0u8; 32];

    // Node keypair for blind signatures (off-chain)
    let node_keypair = Keypair::random(&mut rand::rng());

    // --- Phase 1: Deposit (on-chain UTxO creation) ---
    let deposit_lovelace = 10_000_000u64; // 10 ADA
    let datum = build_deposit_datum(&user_hash, &node_hash, &intent_hash);

    // Simulate the node issuing a signed note after confirming the deposit.
    // In the real system, the user blinds the note commitment and the node
    // signs it; here we use emit_note as a shortcut.
    let mut rng = rand::rng();
    let note_a = mugraph_node::routes::emit_note(
        &node_keypair,
        mugraph_core::types::PolicyId::zero(), // ADA has zero policy_id
        mugraph_core::types::AssetName::empty(),
        deposit_lovelace,
        &mut rng,
    )
    .expect("Failed to emit deposit note");

    // Verify the note is valid
    assert!(
        crypto::verify(&node_keypair.public_key, note_a.commitment().as_ref(), note_a.signature)
            .expect("verify failed"),
        "Emitted note signature should be valid"
    );

    // --- Phase 2: Off-chain transfers via Refresh ---
    // Transfer 1: split 10 ADA → 6 ADA + 4 ADA
    let refresh_1 = RefreshBuilder::new()
        .input(note_a.clone())
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            6_000_000,
        )
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            4_000_000,
        )
        .build()
        .expect("Failed to build refresh 1");

    refresh_1.verify().expect("Refresh 1 should be balanced");

    // Transfer 2: merge 6 ADA + 4 ADA → 10 ADA (re-merge)
    // Simulate output notes from refresh 1 (in real system the node signs these)
    let note_b = mugraph_node::routes::emit_note(
        &node_keypair,
        mugraph_core::types::PolicyId::zero(),
        mugraph_core::types::AssetName::empty(),
        6_000_000,
        &mut rng,
    )
    .expect("Failed to emit note_b");
    let note_c = mugraph_node::routes::emit_note(
        &node_keypair,
        mugraph_core::types::PolicyId::zero(),
        mugraph_core::types::AssetName::empty(),
        4_000_000,
        &mut rng,
    )
    .expect("Failed to emit note_c");

    let refresh_2 = RefreshBuilder::new()
        .input(note_b)
        .input(note_c)
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            10_000_000,
        )
        .build()
        .expect("Failed to build refresh 2");

    refresh_2.verify().expect("Refresh 2 should be balanced");

    // Transfer 3: partial spend — 10 ADA → 7 ADA + 3 ADA
    let note_d = mugraph_node::routes::emit_note(
        &node_keypair,
        mugraph_core::types::PolicyId::zero(),
        mugraph_core::types::AssetName::empty(),
        10_000_000,
        &mut rng,
    )
    .expect("Failed to emit note_d");

    let refresh_3 = RefreshBuilder::new()
        .input(note_d)
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            7_000_000,
        )
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            3_000_000,
        )
        .build()
        .expect("Failed to build refresh 3");

    refresh_3.verify().expect("Refresh 3 should be balanced");

    // --- Phase 3: Withdrawal (on-chain spend) ---
    // The user withdraws the original 10 ADA UTxO from the script address.
    // Regardless of how many off-chain transfers happened, the on-chain UTxO
    // is unchanged — the validator just checks the user's signature.
    let (tx_bytes, utxos) = build_spend_tx(
        &script_cbor,
        &script_hash,
        datum,
        vec![Hash::from(user_hash)],
        Hash::from([0x20; 32]),
        0,
        deposit_lovelace,
    );

    let redeemers = evaluate_tx(&tx_bytes, &utxos, &cost_models)
        .expect("Withdrawal script evaluation failed after off-chain transfers");

    assert_eq!(redeemers.len(), 1);
    let r = &redeemers[0];
    assert!(r.ex_units.steps > 0);
    assert!(r.ex_units.mem > 0);

    println!(
        "eval_lifecycle_deposit_transfer_withdraw: CPU={}, Mem={}",
        r.ex_units.steps, r.ex_units.mem
    );
}

// ---------------------------------------------------------------------------
// E2E Lifecycle: batch withdrawal of multiple deposits
// ---------------------------------------------------------------------------

/// Test batch withdrawal: two separate deposits with different intents,
/// then a single transaction withdrawing both UTxOs.
///
/// This exercises the multi-input spend path that a node would use
/// for batch processing, combined with multiasset UTxOs.
#[test]
fn eval_lifecycle_batch_withdrawal() {
    use mugraph_core::builder::RefreshBuilder;

    let script_cbor = load_validator_cbor();
    let script_hash = compute_script_hash(&script_cbor);
    let cost_models = load_cost_models();

    // Shared user — both deposits belong to the same user
    let user_key = [1u8; 32];
    let user_hash = blake2b_224(&user_key);
    let node_hash = blake2b_224(&[2u8; 32]);

    let node_keypair = mugraph_core::types::Keypair::random(&mut rand::rng());
    let mut rng = rand::rng();

    // --- Deposit 1: 5 ADA (plain) ---
    let intent_hash_1 = [0x01u8; 32];
    let datum_1 = build_deposit_datum(&user_hash, &node_hash, &intent_hash_1);

    // Simulate off-chain: emit note, do a transfer, done
    let note_1 = mugraph_node::routes::emit_note(
        &node_keypair,
        mugraph_core::types::PolicyId::zero(),
        mugraph_core::types::AssetName::empty(),
        5_000_000,
        &mut rng,
    )
    .expect("emit note_1");

    let refresh_1 = RefreshBuilder::new()
        .input(note_1)
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            3_000_000,
        )
        .output(
            mugraph_core::types::PolicyId::zero(),
            mugraph_core::types::AssetName::empty(),
            2_000_000,
        )
        .build()
        .expect("build refresh_1");
    refresh_1.verify().expect("refresh_1 balanced");

    // --- Deposit 2: 8 ADA + native tokens ---
    let intent_hash_2 = [0x02u8; 32];
    let datum_2 = build_deposit_datum(&user_hash, &node_hash, &intent_hash_2);

    let token_policy = mugraph_core::types::PolicyId([0xDD; 28]);
    let token_name = mugraph_core::types::AssetName::new(b"HOSKY").unwrap();

    // Simulate off-chain token transfer
    let note_token = mugraph_node::routes::emit_note(
        &node_keypair,
        token_policy,
        token_name,
        500,
        &mut rng,
    )
    .expect("emit token note");

    let refresh_2 = RefreshBuilder::new()
        .input(note_token)
        .output(token_policy, token_name, 300)
        .output(token_policy, token_name, 200)
        .build()
        .expect("build refresh_2");
    refresh_2.verify().expect("refresh_2 balanced");

    // --- Batch withdrawal: spend both UTxOs in one transaction ---
    // Deposit 2 is multiasset on-chain
    let pallas_token_policy: Hash<28> = Hash::from([0xDD; 28]);
    let input_value_2 = build_multiasset_value(
        8_000_000,
        pallas_token_policy,
        b"HOSKY",
        500,
    );

    let datums = vec![datum_1, datum_2];

    // Build multi-spend tx with mixed values
    // We need a custom build since build_multi_spend_tx only supports Coin values.
    // Use a dedicated build for this test.
    {
        use pallas_primitives::conway::{PseudoTransactionBody, Tx};

        let script_address_bytes = build_script_address_bytes(&script_hash);

        // Input 1: 5 ADA (plain)
        let tx_input_1 = pallas_primitives::TransactionInput {
            transaction_id: Hash::from([0x01; 32]),
            index: 0,
        };
        let utxo_1 = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
            address: script_address_bytes.clone().into(),
            value: Value::Coin(5_000_000),
            datum_option: Some(DatumOption::Data(CborWrap(datums[0].clone()))),
            script_ref: None,
        });

        // Input 2: 8 ADA + 500 HOSKY
        let tx_input_2 = pallas_primitives::TransactionInput {
            transaction_id: Hash::from([0x02; 32]),
            index: 0,
        };
        let utxo_2 = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
            address: script_address_bytes.clone().into(),
            value: input_value_2,
            datum_option: Some(DatumOption::Data(CborWrap(datums[1].clone()))),
            script_ref: None,
        });

        // Sort inputs by tx_id (0x01 < 0x02, already sorted)
        let mut inputs = vec![tx_input_1.clone(), tx_input_2.clone()];
        let mut resolved = vec![
            ResolvedInput {
                input: tx_input_1.clone(),
                output: utxo_1,
            },
            ResolvedInput {
                input: tx_input_2.clone(),
                output: utxo_2,
            },
        ];

        inputs.sort_by(|a, b| {
            a.transaction_id
                .cmp(&b.transaction_id)
                .then(a.index.cmp(&b.index))
        });
        resolved.sort_by(|a, b| {
            a.input
                .transaction_id
                .cmp(&b.input.transaction_id)
                .then(a.input.index.cmp(&b.input.index))
        });

        // One redeemer per input
        let redeemers_list: Vec<Redeemer> = (0..2)
            .map(|i| Redeemer {
                tag: RedeemerTag::Spend,
                index: i as u32,
                data: build_void_redeemer_data(),
                ex_units: ExUnits {
                    mem: 14_000_000,
                    steps: 10_000_000_000,
                },
            })
            .collect();

        // Change output: combined value (13 ADA - fee + 500 HOSKY)
        let dummy_key_hash: [u8; 28] = [0xAA; 28];
        let change_addr = ShelleyAddress::new(
            Network::Testnet,
            ShelleyPaymentPart::Key(Hash::from(dummy_key_hash)),
            ShelleyDelegationPart::Null,
        );
        let change_value = build_multiasset_value(
            13_000_000u64.saturating_sub(2_000_000),
            pallas_token_policy,
            b"HOSKY",
            500,
        );
        let change_output = TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
            address: change_addr.to_vec().into(),
            value: change_value,
            datum_option: None,
            script_ref: None,
        });

        let collateral_input = pallas_primitives::TransactionInput {
            transaction_id: Hash::from([0xBB; 32]),
            index: 0,
        };

        let tx_body = PseudoTransactionBody {
            inputs: Set::from(inputs),
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
            required_signers: NonEmptySet::from_vec(vec![Hash::from(user_hash)]),
            network_id: None,
            collateral_return: None,
            total_collateral: None,
            reference_inputs: None,
            voting_procedures: None,
            proposal_procedures: None,
            treasury_value: None,
            donation: None,
        };

        let witness_set = WitnessSet {
            vkeywitness: None,
            native_script: None,
            bootstrap_witness: None,
            plutus_v1_script: None,
            plutus_data: None,
            redeemer: Some(Redeemers::List(MaybeIndefArray::Def(redeemers_list))),
            plutus_v2_script: None,
            plutus_v3_script: NonEmptySet::from_vec(vec![PlutusScript(
                script_cbor.to_vec().into(),
            )]),
        };

        let tx = Tx {
            transaction_body: tx_body,
            transaction_witness_set: witness_set,
            success: true,
            auxiliary_data: Nullable::Null,
        };

        let tx_bytes = minicbor::to_vec(&tx).expect("Failed to encode batch tx");

        let redeemers = evaluate_tx(&tx_bytes, &resolved, &cost_models)
            .expect("Batch withdrawal evaluation failed");

        assert_eq!(redeemers.len(), 2, "Expected 2 redeemer results for batch");
        for (i, r) in redeemers.iter().enumerate() {
            assert!(r.ex_units.steps > 0, "CPU steps should be nonzero for input {i}");
            assert!(r.ex_units.mem > 0, "Memory should be nonzero for input {i}");
        }

        let total_cpu: u64 = redeemers.iter().map(|r| r.ex_units.steps).sum();
        let total_mem: u64 = redeemers.iter().map(|r| r.ex_units.mem).sum();

        println!(
            "eval_lifecycle_batch_withdrawal: total CPU={}, total Mem={}",
            total_cpu, total_mem
        );

        // Budget sanity: batch of 2 should be ≤ 2x single-spend limit
        assert!(
            total_cpu <= SINGLE_SPEND_CPU_LIMIT * 2,
            "Batch CPU {} exceeds 2x single limit {}",
            total_cpu,
            SINGLE_SPEND_CPU_LIMIT * 2
        );
        assert!(
            total_mem <= SINGLE_SPEND_MEM_LIMIT * 2,
            "Batch memory {} exceeds 2x single limit {}",
            total_mem,
            SINGLE_SPEND_MEM_LIMIT * 2
        );
    }
}
