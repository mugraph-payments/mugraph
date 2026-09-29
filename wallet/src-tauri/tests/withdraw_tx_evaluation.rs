//! Runs the withdrawal transaction that the wallet builds through the
//! Plutus evaluator, with the real deposit validator. Phase one checks
//! that the scripts and redeemers are complete. Phase two runs the
//! validator and checks that the declared execution units are enough.

use mugraph_wallet_lib::{
    cardano_tx::{
        CollateralInput, ProtocolParams, WithdrawTxParams, build_withdraw_tx,
        derive_address,
    },
    cip8::blake2b_224,
};
use pallas_codec::minicbor;
use pallas_primitives::conway::{
    CostModels, TransactionInput, TransactionOutput,
};
use pallas_traverse::{Era, MultiEraTx};
use uplc::{
    machine::cost_model::ExBudget,
    tx::{
        eval_phase_two,
        script_context::{ResolvedInput, SlotConfig},
    },
};
use whisky_csl::csl;

fn validator_cbor() -> Vec<u8> {
    let plutus: serde_json::Value =
        serde_json::from_str(include_str!("../../../validator/plutus.json"))
            .unwrap();
    hex::decode(plutus["validators"][0]["compiledCode"].as_str().unwrap())
        .unwrap()
}

fn cost_model_v3() -> Vec<i64> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../node/tests/fixtures/preprod_cost_models.json"
    ))
    .unwrap();
    fixture["PlutusV3"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect()
}

fn protocol() -> ProtocolParams {
    ProtocolParams {
        min_fee_a: 44,
        min_fee_b: 155_381,
        price_mem: 0.0577,
        price_step: 0.0000721,
        plutus_v3_cost_model: cost_model_v3(),
    }
}

fn script_address(script_cbor: &[u8]) -> String {
    let hash = csl::PlutusScript::new_v3(script_cbor.to_vec()).hash();
    csl::EnterpriseAddress::new(0, &csl::Credential::from_scripthash(&hash))
        .to_address()
        .to_bech32(None)
        .unwrap()
}

fn vault_datum(node_hash: &[u8; 28]) -> csl::PlutusData {
    let mut fields = csl::PlutusList::new();
    fields.add(&csl::PlutusData::new_bytes(vec![1u8; 28]));
    fields.add(&csl::PlutusData::new_bytes(node_hash.to_vec()));
    fields.add(&csl::PlutusData::new_bytes(vec![0u8; 32]));
    csl::PlutusData::new_constr_plutus_data(&csl::ConstrPlutusData::new(
        &csl::BigNum::zero(),
        &fields,
    ))
}

fn resolved(
    tx_hash: &str,
    index: u32,
    output: &csl::TransactionOutput,
) -> ResolvedInput {
    let input = csl::TransactionInput::new(
        &csl::TransactionHash::from_hex(tx_hash).unwrap(),
        index,
    );
    ResolvedInput {
        input: minicbor::decode::<TransactionInput>(&input.to_bytes()).unwrap(),
        output: minicbor::decode::<TransactionOutput>(&output.to_bytes())
            .unwrap(),
    }
}

#[test]
fn withdraw_tx_passes_the_deposit_validator() {
    let script = validator_cbor();
    let vault = script_address(&script);
    let node_hash = blake2b_224(&[6u8; 32]);
    let user_vk = [7u8; 32];
    let user_address = derive_address(&user_vk, "preprod").unwrap();

    // Two vault UTxOs, so the redeemer indexes matter.
    let vault_inputs = [("bb".repeat(32), 1u32), ("aa".repeat(32), 0u32)];
    let params = protocol();
    let collateral = CollateralInput {
        tx_hash: "cc".repeat(32),
        index: 0,
        lovelace: 5_000_000,
        return_address: user_address.clone(),
    };

    let built = build_withdraw_tx(&WithdrawTxParams {
        script_inputs: &vault_inputs,
        total_input_lovelace: 20_000_000,
        destination_address: &user_address,
        withdraw_amount_lovelace: 5_000_000,
        script_address: &vault,
        node_pubkey_hash: &node_hash,
        user_pubkey_hash: &blake2b_224(&user_vk),
        script_cbor: &script,
        collateral: &collateral,
        protocol: &params,
    })
    .unwrap();

    // The fee must cover the size and the declared execution units.
    let tx = csl::Transaction::from_bytes(built.tx_cbor.clone()).unwrap();
    let body = tx.body();
    assert_eq!(body.fee().to_str(), built.fee.to_string());
    assert!(body.script_data_hash().is_some());
    assert!(body.collateral().is_some());
    assert!(body.collateral_return().is_some());
    let redeemers = tx.witness_set().redeemers().unwrap();
    assert_eq!(redeemers.len(), 2);
    let size = built.tx_cbor.len() as u64 + 2 * 101; // two vkey witnesses to add
    let mut script_fee = 0f64;
    for i in 0..redeemers.len() {
        let units = redeemers.get(i).ex_units();
        script_fee += units.mem().to_str().parse::<f64>().unwrap()
            * params.price_mem
            + units.steps().to_str().parse::<f64>().unwrap()
                * params.price_step;
    }
    assert!(
        built.fee
            >= params.min_fee_a * size
                + params.min_fee_b
                + script_fee.ceil() as u64,
        "fee {} is below the minimum",
        built.fee
    );

    // The UTxOs that the transaction spends.
    let vault_output = |lovelace: u64| {
        let mut output = csl::TransactionOutput::new(
            &csl::Address::from_bech32(&vault).unwrap(),
            &csl::Value::new(
                &csl::Coin::from_str(&lovelace.to_string()).unwrap(),
            ),
        );
        output.set_plutus_data(&vault_datum(&node_hash));
        output
    };
    let collateral_output = csl::TransactionOutput::new(
        &csl::Address::from_bech32(&user_address).unwrap(),
        &csl::Value::new(&csl::Coin::from_str("5000000").unwrap()),
    );
    let utxos = vec![
        resolved(
            &vault_inputs[0].0,
            vault_inputs[0].1,
            &vault_output(12_000_000),
        ),
        resolved(
            &vault_inputs[1].0,
            vault_inputs[1].1,
            &vault_output(8_000_000),
        ),
        resolved(&collateral.tx_hash, collateral.index, &collateral_output),
    ];

    let multi =
        MultiEraTx::decode_for_era(Era::Conway, &built.tx_cbor).unwrap();
    let MultiEraTx::Conway(minted) = &multi else {
        panic!("expected a Conway transaction");
    };
    let cost_models = CostModels {
        plutus_v1: None,
        plutus_v2: None,
        plutus_v3: Some(cost_model_v3()),
    };
    let evaluated = eval_phase_two(
        minted,
        &utxos,
        Some(&cost_models),
        Some(&ExBudget {
            cpu: 10_000_000_000,
            mem: 14_000_000,
        }),
        &SlotConfig {
            zero_time: 1655683200000,
            zero_slot: 0,
            slot_length: 1000,
        },
        true,
        |_| (),
    )
    .expect("the validator must accept the withdrawal");

    assert_eq!(evaluated.len(), 2);
    for (declared, used) in (0..redeemers.len())
        .map(|i| redeemers.get(i).ex_units())
        .zip(&evaluated)
    {
        assert!(
            used.ex_units.mem
                <= declared.mem().to_str().parse::<u64>().unwrap()
        );
        assert!(
            used.ex_units.steps
                <= declared.steps().to_str().parse::<u64>().unwrap()
        );
    }
}
