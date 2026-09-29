//! The ledger rules that the mock chain checks before it accepts a
//! transaction. They are a subset of the Cardano ledger rules, chosen so
//! that a transaction that the mock chain accepts has a good chance to
//! pass on a real network:
//!
//! - each vkey witness is a valid signature of the transaction body;
//! - each key-locked input and collateral input has a witness;
//! - each required signer has a witness;
//! - each script input has its script and a spend redeemer, and the
//!   script passes with the declared execution units (phase two);
//! - the inputs pay for the outputs and the fee (ADA only);
//! - the fee is at least the minimum fee for the size and the scripts.
//!
//! It does not check the script data hash, validity intervals, minimum
//! UTxO values, or native assets.

use std::collections::{BTreeMap, HashSet};

use pallas_addresses::{Address, ShelleyPaymentPart};
use pallas_codec::{minicbor, utils::CborWrap};
use pallas_crypto::{
    hash::Hasher,
    key::ed25519::{PublicKey, Signature},
};
use pallas_primitives::conway::{
    CostModels, DatumOption, PlutusData, PostAlonzoTransactionOutput,
    RedeemerTag, Redeemers, TransactionInput, TransactionOutput, Value,
};
use pallas_traverse::{Era, MultiEraTx};
use uplc::{
    machine::cost_model::ExBudget,
    tx::{
        eval_phase_two,
        script_context::{ResolvedInput, SlotConfig},
    },
};

use crate::state::UtxoEntry;

/// Linear fee parameters (preprod values).
pub const MIN_FEE_A: u64 = 44;
pub const MIN_FEE_B: u64 = 155_381;
pub const PRICE_MEM: f64 = 0.0577;
pub const PRICE_STEP: f64 = 0.0000721;

/// Checks `tx_cbor` against the ledger rules above. `resolve` gives the
/// live UTxO for an input, or `None` if it does not exist.
pub fn validate_tx(
    tx_cbor: &[u8],
    resolve: impl Fn(&str, u16) -> Option<UtxoEntry>,
) -> Result<(), String> {
    let multi = MultiEraTx::decode_for_era(Era::Conway, tx_cbor)
        .map_err(|e| format!("decode: {e}"))?;
    let tx = multi.as_conway().ok_or("expected a Conway transaction")?;
    let body = &tx.transaction_body;
    let witnesses = &tx.transaction_witness_set;
    let tx_hash = multi.hash();

    // Vkey witnesses must sign the body.
    let mut signed: HashSet<[u8; 28]> = HashSet::new();
    for witness in witnesses.vkeywitness.iter().flat_map(|w| w.iter()) {
        let vkey: [u8; 32] = witness
            .vkey
            .as_slice()
            .try_into()
            .map_err(|_| "bad vkey length")?;
        let signature: [u8; 64] = witness
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| "bad signature length")?;
        if !PublicKey::from(vkey).verify(tx_hash, &Signature::from(signature)) {
            return Err("invalid vkey witness".to_string());
        }
        signed.insert(*Hasher::<224>::hash(&vkey));
    }

    // Scripts and spend redeemers in the witness set.
    let scripts: HashSet<[u8; 28]> = witnesses
        .plutus_v3_script
        .iter()
        .flat_map(|s| s.iter())
        .map(|script| *Hasher::<224>::hash_tagged(script.as_ref(), 3))
        .collect();
    let mut spend_units: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
    let mut total_units = (0u64, 0u64);
    match witnesses.redeemer.as_deref() {
        Some(Redeemers::List(list)) => {
            for r in list.iter() {
                total_units.0 += r.ex_units.mem;
                total_units.1 += r.ex_units.steps;
                if r.tag == RedeemerTag::Spend {
                    spend_units
                        .insert(r.index, (r.ex_units.mem, r.ex_units.steps));
                }
            }
        }
        Some(Redeemers::Map(map)) => {
            for (k, v) in map.iter() {
                total_units.0 += v.ex_units.mem;
                total_units.1 += v.ex_units.steps;
                if k.tag == RedeemerTag::Spend {
                    spend_units
                        .insert(k.index, (v.ex_units.mem, v.ex_units.steps));
                }
            }
        }
        None => {}
    }

    // Inputs, in ledger order, since spend redeemers point by that order.
    let mut inputs: Vec<&TransactionInput> = body.inputs.iter().collect();
    inputs.sort_by(|a, b| {
        (a.transaction_id, a.index).cmp(&(b.transaction_id, b.index))
    });

    let mut resolved = Vec::with_capacity(inputs.len());
    let mut input_lovelace = 0u64;
    let mut runs_scripts = false;
    for (i, input) in inputs.iter().enumerate() {
        let utxo =
            resolve(&hex::encode(input.transaction_id), input.index as u16)
                .ok_or_else(|| format!("input {} not found", i))?;
        input_lovelace += utxo.lovelace;

        match payment_part(&utxo.address)? {
            ShelleyPaymentPart::Key(hash) => {
                if !signed.contains(&*hash) {
                    return Err(format!("missing witness for input {}", i));
                }
            }
            ShelleyPaymentPart::Script(hash) => {
                if !scripts.contains(&*hash) {
                    return Err(format!("missing script for input {}", i));
                }
                if !spend_units.contains_key(&(i as u32)) {
                    return Err(format!(
                        "missing spend redeemer for input {}",
                        i
                    ));
                }
                runs_scripts = true;
            }
        }

        resolved.push(ResolvedInput {
            input: (*input).clone(),
            output: resolved_output(&utxo)?,
        });
    }

    // Collateral must be locked by keys that signed.
    for input in body.collateral.iter().flat_map(|c| c.iter()) {
        let utxo =
            resolve(&hex::encode(input.transaction_id), input.index as u16)
                .ok_or("collateral input not found")?;
        match payment_part(&utxo.address)? {
            ShelleyPaymentPart::Key(hash) if signed.contains(&*hash) => {}
            _ => return Err("missing witness for collateral input".to_string()),
        }
    }

    for signer in body.required_signers.iter().flat_map(|s| s.iter()) {
        if !signed.contains(&**signer) {
            return Err(format!(
                "missing witness for required signer {}",
                hex::encode(signer)
            ));
        }
    }

    // ADA balance.
    let output_lovelace: u64 =
        multi.outputs().iter().map(|o| o.value().coin()).sum();
    if input_lovelace != output_lovelace + body.fee {
        return Err(format!(
            "balance: inputs {} != outputs {} + fee {}",
            input_lovelace, output_lovelace, body.fee
        ));
    }

    // Minimum fee.
    let script_fee = (total_units.0 as f64 * PRICE_MEM
        + total_units.1 as f64 * PRICE_STEP)
        .ceil() as u64;
    let min_fee = MIN_FEE_A * tx_cbor.len() as u64 + MIN_FEE_B + script_fee;
    if body.fee < min_fee {
        return Err(format!(
            "fee {} is below the minimum {}",
            body.fee, min_fee
        ));
    }

    // Phase two: run the scripts.
    if runs_scripts {
        let evaluated = eval_phase_two(
            tx,
            &resolved,
            Some(&cost_models()),
            Some(&ExBudget {
                cpu: 10_000_000_000,
                mem: 14_000_000,
            }),
            &SlotConfig {
                zero_time: 1655683200000,
                zero_slot: 0,
                slot_length: 1000,
            },
            false,
            |_| (),
        )
        .map_err(|e| format!("script failed: {e}"))?;

        for r in evaluated {
            if r.tag != RedeemerTag::Spend {
                continue;
            }
            let declared = spend_units.get(&r.index).copied().unwrap_or((0, 0));
            if r.ex_units.mem > declared.0 || r.ex_units.steps > declared.1 {
                return Err(format!(
                    "script for input {} uses more than its declared execution units",
                    r.index
                ));
            }
        }
    }

    Ok(())
}

fn payment_part(address: &str) -> Result<ShelleyPaymentPart, String> {
    match Address::from_bech32(address)
        .map_err(|e| format!("address {address}: {e}"))?
    {
        Address::Shelley(shelley) => Ok(shelley.payment().clone()),
        _ => Err(format!(
            "address {address} is not a Shelley payment address"
        )),
    }
}

fn resolved_output(utxo: &UtxoEntry) -> Result<TransactionOutput, String> {
    let address = Address::from_bech32(&utxo.address)
        .map_err(|e| format!("address {}: {e}", utxo.address))?;
    let datum_option = match &utxo.inline_datum_cbor {
        Some(cbor) => {
            let bytes =
                hex::decode(cbor).map_err(|e| format!("datum hex: {e}"))?;
            let data: PlutusData = minicbor::decode(&bytes)
                .map_err(|e| format!("datum cbor: {e}"))?;
            Some(DatumOption::Data(CborWrap(data)))
        }
        None => None,
    };

    Ok(TransactionOutput::PostAlonzo(PostAlonzoTransactionOutput {
        address: address.to_vec().into(),
        value: Value::Coin(utxo.lovelace),
        datum_option,
        script_ref: None,
    }))
}

fn cost_models() -> CostModels {
    let json: serde_json::Value =
        serde_json::from_str(include_str!("cost_models.json"))
            .expect("cost model fixture is valid JSON");
    CostModels {
        plutus_v1: None,
        plutus_v2: None,
        plutus_v3: Some(
            json["PlutusV3"]
                .as_array()
                .expect("fixture has PlutusV3")
                .iter()
                .map(|v| v.as_i64().expect("cost is an integer"))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use whisky_csl::csl;

    use super::*;

    fn key(seed: u8) -> csl::PrivateKey {
        csl::PrivateKey::from_normal_bytes(&[seed; 32]).unwrap()
    }

    fn key_address(seed: u8) -> String {
        let hash = key(seed).to_public().hash();
        csl::EnterpriseAddress::new(0, &csl::Credential::from_keyhash(&hash))
            .to_address()
            .to_bech32(None)
            .unwrap()
    }

    fn validator() -> Vec<u8> {
        let plutus: serde_json::Value =
            serde_json::from_str(include_str!("../../validator/plutus.json"))
                .unwrap();
        hex::decode(plutus["validators"][0]["compiledCode"].as_str().unwrap())
            .unwrap()
    }

    fn vault_address() -> String {
        let hash = csl::PlutusScript::new_v3(validator()).hash();
        csl::EnterpriseAddress::new(0, &csl::Credential::from_scripthash(&hash))
            .to_address()
            .to_bech32(None)
            .unwrap()
    }

    fn vault_datum_hex(node_seed: u8) -> String {
        let mut fields = csl::PlutusList::new();
        fields.add(&csl::PlutusData::new_bytes(vec![1u8; 28]));
        fields.add(&csl::PlutusData::new_bytes(
            key(node_seed).to_public().hash().to_bytes(),
        ));
        fields.add(&csl::PlutusData::new_bytes(vec![0u8; 32]));
        hex::encode(
            csl::PlutusData::new_constr_plutus_data(
                &csl::ConstrPlutusData::new(&csl::BigNum::zero(), &fields),
            )
            .to_bytes(),
        )
    }

    fn utxo(
        tx: &str,
        address: &str,
        lovelace: u64,
        datum: Option<String>,
    ) -> UtxoEntry {
        UtxoEntry {
            tx_hash: tx.to_string(),
            output_index: 0,
            address: address.to_string(),
            lovelace,
            datum_hash: None,
            inline_datum_cbor: datum,
            block_height: Some(1),
        }
    }

    fn resolver(
        utxos: Vec<UtxoEntry>,
    ) -> impl Fn(&str, u16) -> Option<UtxoEntry> {
        move |hash, index| {
            utxos
                .iter()
                .find(|u| u.tx_hash == hash && u.output_index == index)
                .cloned()
        }
    }

    /// A transaction from one input to one output, signed by `signers`.
    struct Spec {
        input: String,
        output: u64,
        fee: u64,
        required: Vec<u8>,
        signers: Vec<u8>,
        /// Spend the input with the validator: (collateral input, redeemer?)
        script: Option<(String, bool)>,
    }

    fn build(spec: &Spec) -> Vec<u8> {
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&csl::TransactionInput::new(
            &csl::TransactionHash::from_hex(&spec.input).unwrap(),
            0,
        ));
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&csl::TransactionOutput::new(
            &csl::Address::from_bech32(&key_address(9)).unwrap(),
            &csl::Value::new(&csl::BigNum::from(spec.output)),
        ));
        let mut body = csl::TransactionBody::new_tx_body(
            &inputs,
            &outputs,
            &csl::BigNum::from(spec.fee),
        );
        if !spec.required.is_empty() {
            let mut required = csl::Ed25519KeyHashes::new();
            for seed in &spec.required {
                required.add(&key(*seed).to_public().hash());
            }
            body.set_required_signers(&required);
        }

        let mut witness_set = csl::TransactionWitnessSet::new();
        if let Some((collateral, with_redeemer)) = &spec.script {
            let mut collateral_inputs = csl::TransactionInputs::new();
            collateral_inputs.add(&csl::TransactionInput::new(
                &csl::TransactionHash::from_hex(collateral).unwrap(),
                0,
            ));
            body.set_collateral(&collateral_inputs);

            let mut scripts = csl::PlutusScripts::new();
            scripts.add(&csl::PlutusScript::new_v3(validator()));
            witness_set.set_plutus_scripts(&scripts);
            if *with_redeemer {
                let mut redeemers = csl::Redeemers::new();
                redeemers.add(&csl::Redeemer::new(
                    &csl::RedeemerTag::new_spend(),
                    &csl::BigNum::zero(),
                    &csl::PlutusData::new_empty_constr_plutus_data(
                        &csl::BigNum::zero(),
                    ),
                    &csl::ExUnits::new(
                        &csl::BigNum::from(100_000u64),
                        &csl::BigNum::from(40_000_000u64),
                    ),
                ));
                witness_set.set_redeemers(&redeemers);
            }
        }

        let hash = csl::TransactionHash::from_bytes(
            blake2b_256(&body.to_bytes()).to_vec(),
        )
        .unwrap();
        if !spec.signers.is_empty() {
            let mut vkeys = csl::Vkeywitnesses::new();
            for seed in &spec.signers {
                vkeys.add(&csl::make_vkey_witness(&hash, &key(*seed)));
            }
            witness_set.set_vkeys(&vkeys);
        }

        csl::Transaction::new(&body, &witness_set, None).to_bytes()
    }

    fn blake2b_256(data: &[u8]) -> [u8; 32] {
        use blake2::{Blake2b, Digest, digest::consts::U32};
        let mut out = [0u8; 32];
        out.copy_from_slice(&Blake2b::<U32>::digest(data));
        out
    }

    fn key_spend(signers: Vec<u8>) -> Spec {
        Spec {
            input: "aa".repeat(32),
            output: 9_700_000,
            fee: 300_000,
            required: vec![],
            signers,
            script: None,
        }
    }

    fn key_utxos() -> Vec<UtxoEntry> {
        vec![utxo(&"aa".repeat(32), &key_address(1), 10_000_000, None)]
    }

    #[test]
    fn accepts_a_key_input_with_its_witness() {
        let tx = build(&key_spend(vec![1]));
        validate_tx(&tx, resolver(key_utxos())).unwrap();
    }

    #[test]
    fn rejects_a_key_input_without_its_witness() {
        let tx = build(&key_spend(vec![2]));
        let err = validate_tx(&tx, resolver(key_utxos())).unwrap_err();
        assert!(err.contains("witness"), "{err}");
    }

    #[test]
    fn rejects_a_missing_input() {
        let tx = build(&key_spend(vec![1]));
        let err = validate_tx(&tx, resolver(vec![])).unwrap_err();
        assert!(err.contains("not found"), "{err}");
    }

    #[test]
    fn rejects_an_unbalanced_transaction() {
        let tx = build(&Spec {
            output: 9_800_000,
            ..key_spend(vec![1])
        });
        let err = validate_tx(&tx, resolver(key_utxos())).unwrap_err();
        assert!(err.contains("balance"), "{err}");
    }

    #[test]
    fn rejects_a_fee_below_the_minimum() {
        let tx = build(&Spec {
            output: 9_900_000,
            fee: 100_000,
            ..key_spend(vec![1])
        });
        let err = validate_tx(&tx, resolver(key_utxos())).unwrap_err();
        assert!(err.contains("fee"), "{err}");
    }

    #[test]
    fn rejects_a_required_signer_without_a_witness() {
        let tx = build(&Spec {
            required: vec![5],
            ..key_spend(vec![1])
        });
        let err = validate_tx(&tx, resolver(key_utxos())).unwrap_err();
        assert!(err.contains("required signer"), "{err}");
    }

    fn script_utxos() -> Vec<UtxoEntry> {
        vec![
            utxo(
                &"aa".repeat(32),
                &vault_address(),
                10_000_000,
                Some(vault_datum_hex(6)),
            ),
            utxo(&"cc".repeat(32), &key_address(1), 5_000_000, None),
        ]
    }

    fn script_spend(with_redeemer: bool, node_signs: bool) -> Spec {
        Spec {
            input: "aa".repeat(32),
            output: 9_500_000,
            fee: 500_000,
            required: if node_signs { vec![6] } else { vec![] },
            signers: if node_signs { vec![1, 6] } else { vec![1] },
            script: Some(("cc".repeat(32), with_redeemer)),
        }
    }

    #[test]
    fn accepts_a_script_spend_that_the_validator_allows() {
        let tx = build(&script_spend(true, true));
        validate_tx(&tx, resolver(script_utxos())).unwrap();
    }

    #[test]
    fn rejects_a_script_spend_without_a_redeemer() {
        let tx = build(&script_spend(false, true));
        let err = validate_tx(&tx, resolver(script_utxos())).unwrap_err();
        assert!(err.contains("redeemer"), "{err}");
    }

    #[test]
    fn rejects_a_script_spend_that_the_validator_refuses() {
        let tx = build(&script_spend(true, false));
        let err = validate_tx(&tx, resolver(script_utxos())).unwrap_err();
        assert!(err.contains("script"), "{err}");
    }
}
