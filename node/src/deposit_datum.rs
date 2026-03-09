use mugraph_core::error::Error;
use whisky_csl::csl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositDatum {
    pub user_pubkey_hash: [u8; 28],
    pub node_pubkey_hash: [u8; 28],
    pub intent_hash: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositDatumContext {
    DepositUtxo,
    WithdrawalInput { input_index: usize },
}

impl DepositDatumContext {
    fn invalid_hex(self, err: &impl std::fmt::Display) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => format!("Invalid datum hex: {}", err),
                Self::WithdrawalInput { input_index } => {
                    format!("Invalid datum hex for input {}: {}", input_index, err)
                }
            },
        }
    }

    fn invalid_cbor(self, err: &impl std::fmt::Display) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => format!("Invalid datum CBOR: {}", err),
                Self::WithdrawalInput { input_index } => {
                    format!("Invalid datum CBOR for input {}: {}", input_index, err)
                }
            },
        }
    }

    fn not_constructor(self) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => "Datum is not a constructor".to_string(),
                Self::WithdrawalInput { input_index } => {
                    format!("Datum for input {} is not a constructor as expected", input_index)
                }
            },
        }
    }

    fn unexpected_constructor(self, actual: &str) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => {
                    format!("Unexpected datum constructor {}, expected 0", actual)
                }
                Self::WithdrawalInput { input_index } => format!(
                    "Unexpected datum constructor {} for input {} (expected 0)",
                    actual, input_index
                ),
            },
        }
    }

    fn wrong_field_count(self, actual: usize) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => format!("Datum has {} fields (expected 3)", actual),
                Self::WithdrawalInput { input_index } => format!(
                    "Datum for input {} has {} fields (expected 3)",
                    input_index, actual
                ),
            },
        }
    }

    fn missing_user_hash(self) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => {
                    "Datum missing user_pubkey_hash bytes".to_string()
                }
                Self::WithdrawalInput { input_index } => format!(
                    "Datum for input {} missing user_pubkey_hash bytes",
                    input_index
                ),
            },
        }
    }

    fn missing_node_hash(self) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => {
                    "Datum missing node_pubkey_hash bytes".to_string()
                }
                Self::WithdrawalInput { input_index } => format!(
                    "Datum for input {} missing node_pubkey_hash bytes",
                    input_index
                ),
            },
        }
    }

    fn missing_intent_hash(self) -> Error {
        Error::InvalidInput {
            reason: match self {
                Self::DepositUtxo => "Datum missing intent_hash bytes".to_string(),
                Self::WithdrawalInput { input_index } => {
                    format!("Datum for input {} missing intent_hash bytes", input_index)
                }
            },
        }
    }

    fn invalid_key_hash_lengths(self, user_len: usize, node_len: usize) -> Error {
        Error::InvalidInput {
            reason: format!(
                "Datum key hash lengths invalid (user {}, node {}, expected 28)",
                user_len, node_len
            ),
        }
    }

    fn invalid_intent_hash_length(self, actual: usize) -> Error {
        Error::InvalidInput {
            reason: format!(
                "Datum intent_hash length invalid ({} bytes, expected 32)",
                actual
            ),
        }
    }
}

pub fn parse_deposit_datum(
    datum_hex: &str,
    context: DepositDatumContext,
) -> Result<DepositDatum, Error> {
    let datum_bytes = hex::decode(datum_hex).map_err(|e| context.invalid_hex(&e))?;
    let pd = csl::PlutusData::from_bytes(datum_bytes).map_err(|e| context.invalid_cbor(&e))?;

    let constr = pd.as_constr_plutus_data().ok_or_else(|| context.not_constructor())?;
    let alternative = constr.alternative().to_str();
    if alternative != "0" {
        return Err(context.unexpected_constructor(&alternative));
    }

    let fields = constr.data();
    if fields.len() != 3 {
        return Err(context.wrong_field_count(fields.len()));
    }

    let user_pubkey_hash = fields
        .get(0)
        .as_bytes()
        .ok_or_else(|| context.missing_user_hash())?;
    let node_pubkey_hash = fields
        .get(1)
        .as_bytes()
        .ok_or_else(|| context.missing_node_hash())?;
    let intent_hash = fields
        .get(2)
        .as_bytes()
        .ok_or_else(|| context.missing_intent_hash())?;

    let user_pubkey_hash_len = user_pubkey_hash.len();
    let node_pubkey_hash_len = node_pubkey_hash.len();
    let intent_hash_len = intent_hash.len();

    let user_pubkey_hash: [u8; 28] = user_pubkey_hash.try_into().map_err(|_| {
        context.invalid_key_hash_lengths(user_pubkey_hash_len, node_pubkey_hash_len)
    })?;
    let node_pubkey_hash: [u8; 28] = node_pubkey_hash.try_into().map_err(|_| {
        context.invalid_key_hash_lengths(user_pubkey_hash_len, node_pubkey_hash_len)
    })?;
    let intent_hash: [u8; 32] = intent_hash
        .try_into()
        .map_err(|_| context.invalid_intent_hash_length(intent_hash_len))?;

    Ok(DepositDatum {
        user_pubkey_hash,
        node_pubkey_hash,
        intent_hash,
    })
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use pallas_codec::minicbor;
    use pallas_primitives::{alonzo::PlutusData, BoundedBytes, Constr, MaybeIndefArray};

    use super::*;

    fn mk_datum_hex(fields: Vec<Vec<u8>>) -> String {
        let datum = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Def(
                fields
                    .into_iter()
                    .map(|v| PlutusData::BoundedBytes(BoundedBytes::from(v)))
                    .collect(),
            ),
        });

        hex::encode(minicbor::to_vec(&datum).expect("encode datum"))
    }

    #[test]
    fn deposit_datum_parser_extracts_user_node_and_intent_bytes() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .expect("valid user key")
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(node_sk.verifying_key().as_bytes())
            .expect("valid node key")
            .hash()
            .to_bytes();
        let intent_hash = [9u8; 32];
        let datum_hex = mk_datum_hex(vec![user_hash.clone(), node_hash.clone(), intent_hash.to_vec()]);

        let parsed = parse_deposit_datum(&datum_hex, DepositDatumContext::DepositUtxo)
            .expect("datum parses");

        assert_eq!(parsed.user_pubkey_hash.as_slice(), user_hash.as_slice());
        assert_eq!(parsed.node_pubkey_hash.as_slice(), node_hash.as_slice());
        assert_eq!(parsed.intent_hash, intent_hash);
    }

    #[test]
    fn deposit_and_withdraw_callers_preserve_distinct_error_paths_after_shared_parse() {
        let deposit_err = parse_deposit_datum("not-hex", DepositDatumContext::DepositUtxo)
            .expect_err("deposit path should fail");
        assert!(format!("{deposit_err:?}").contains("Invalid datum hex:"));

        let withdraw_err = parse_deposit_datum(
            "not-hex",
            DepositDatumContext::WithdrawalInput { input_index: 4 },
        )
        .expect_err("withdraw path should fail");
        assert!(format!("{withdraw_err:?}").contains("Invalid datum hex for input 4:"));
    }
}
