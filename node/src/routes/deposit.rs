use color_eyre::eyre::Result;
#[cfg(test)]
use mugraph_core::types::{PublicKey, UtxoRef};
use mugraph_core::{
    crypto,
    error::Error,
    types::{BlindSignature, DepositRequest, Response},
};
#[cfg(test)]
use whisky_csl::csl;

#[cfg(test)]
use crate::database::{CARDANO_WALLET, DEPOSITS};
use crate::routes::Context;

mod claims;
mod persistence;
mod signature;
mod source_validation;

use self::{
    claims::parse_deposit_claims,
    persistence::{create_provider, load_or_create_wallet, persist_deposit},
    signature::verify_deposit_signature,
    source_validation::validate_deposit_source,
};
#[cfg(test)]
use self::{
    persistence::{insert_deposit_if_absent, store_wallet_if_absent},
    signature::{
        CanonicalPayload,
        CanonicalUtxo,
        build_canonical_payload,
        compute_intent_hash,
        verify_cip8_cose_signature,
    },
    source_validation::{validate_deposit_amounts, validate_deposit_datum},
};

/// Handle deposit request
///
/// 1. Parse and validate the request payload
/// 2. Verify CIP-8 signature
/// 3. Fetch UTxO from provider
/// 4. Validate UTxO is at script address and unspent
/// 5. Map assets and validate amounts
/// 6. Sign blinded outputs
/// 7. Record deposit in database
pub async fn handle_deposit(
    request: &DepositRequest,
    ctx: &Context,
) -> Result<Response, Error> {
    tracing::info!(
        "Processing deposit request for UTxO: {}:{}",
        &request.utxo.tx_hash[..std::cmp::min(16, request.utxo.tx_hash.len())],
        request.utxo.index
    );

    let claims = parse_deposit_claims(request)?;

    // 1. Load or create Cardano wallet
    let wallet = load_or_create_wallet(ctx).await?;

    // 2. Verify CIP-8 signature over canonical payload (strict)
    verify_deposit_signature(
        request,
        &claims,
        &wallet,
        &ctx.keypair.public_key,
    )?;

    // 3. Fetch UTxO from Cardano provider and validate
    let provider = create_provider(ctx)?;
    validate_deposit_source(
        request,
        &claims,
        &wallet,
        &provider,
        ctx,
        &ctx.keypair.public_key,
    )
    .await?;

    // 5. Sign blinded outputs with delegate key
    let signatures = sign_outputs(request, &ctx.keypair)?;

    // 6. Record deposit in database
    let deposit_ref = persist_deposit(request, ctx, &provider, &wallet).await?;

    tracing::info!(
        "Deposit processed successfully: {}",
        &deposit_ref[..std::cmp::min(32, deposit_ref.len())]
    );

    Ok(Response::Deposit {
        signatures,
        deposit_ref,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};

    use coset::{
        CoseSign1,
        CoseSign1Builder,
        Header,
        ProtectedHeader,
        TaggedCborSerializable,
        iana,
    };
    use ed25519_dalek::{Signer, SigningKey};
    use mugraph_core::types::UtxoReference;

    use super::*;

    static SEED_COUNTER: AtomicU8 = AtomicU8::new(1);

    fn gen_key() -> (SigningKey, Vec<u8>) {
        let seed_byte = SEED_COUNTER.fetch_add(1, Ordering::SeqCst);
        let seed = [seed_byte; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk = sk.verifying_key().to_bytes().to_vec();
        (sk, pk)
    }

    fn build_cip8_signature(sk: &SigningKey, payload: &[u8]) -> Vec<u8> {
        build_custom_cip8_signature(
            sk,
            Some(payload),
            Some(coset::RegisteredLabelWithPrivate::Assigned(
                iana::Algorithm::EdDSA,
            )),
            None,
        )
    }

    fn build_custom_cip8_signature(
        sk: &SigningKey,
        payload: Option<&[u8]>,
        alg: Option<coset::RegisteredLabelWithPrivate<iana::Algorithm>>,
        signature_override: Option<Vec<u8>>,
    ) -> Vec<u8> {
        let header = Header {
            alg,
            ..Default::default()
        };
        let unprotected = Header::default();
        let tbs = CoseSign1 {
            protected: ProtectedHeader {
                original_data: None,
                header: header.clone(),
            },
            unprotected,
            payload: payload.map(|bytes| bytes.to_vec()),
            signature: vec![],
        }
        .tbs_data(&[]);
        let signature =
            signature_override.unwrap_or_else(|| sk.sign(&tbs).to_vec());

        let mut builder = CoseSign1Builder::new()
            .protected(header)
            .signature(signature);
        if let Some(payload) = payload {
            builder = builder.payload(payload.to_vec());
        }
        builder.build().to_tagged_vec().unwrap()
    }

    #[test]
    fn test_cip8_verification_succeeds() {
        let (sk, pk_bytes) = gen_key();
        let mut request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.clone().try_into().unwrap()),
            "addr_test1...",
        );
        let sig = build_cip8_signature(&sk, &payload);
        request.signature = sig;

        assert!(verify_cip8_cose_signature(&request, &payload).is_ok());
    }

    #[test]
    fn test_cip8_verification_fails_on_payload_mismatch() {
        let (sk, pk_bytes) = gen_key();
        let mut request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.clone().try_into().unwrap()),
            "addr_test1...",
        );
        let sig = build_cip8_signature(&sk, &payload);

        // mutate payload by changing network after signing
        request.network = "mainnet".to_string();
        request.signature = sig;
        let bad_payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );

        let res = verify_cip8_cose_signature(&request, &bad_payload);
        assert!(res.is_err());
    }

    #[test]
    fn network_wrapper_preserves_canonical_deposit_payload_bytes() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };
        let delegate_pk = PublicKey(pk_bytes.try_into().unwrap());

        let payload =
            build_canonical_payload(&request, &delegate_pk, "addr_test1...");
        let expected = serde_json::to_string(&CanonicalPayload {
            utxo: CanonicalUtxo {
                tx_hash: request.utxo.tx_hash.clone(),
                index: request.utxo.index,
            },
            outputs: vec![],
            delegate_pk: hex::encode(delegate_pk.0),
            script_address: "addr_test1...".to_string(),
            nonce: request.nonce,
            network: request.network.clone(),
        })
        .unwrap()
        .into_bytes();

        assert_eq!(payload, expected);
    }

    #[test]
    fn parse_deposit_claims_decodes_exact_user_pubkey_bytes() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let claims = parse_deposit_claims(&request).expect("claims parse");
        let expected: [u8; 32] = pk_bytes.try_into().expect("32-byte pubkey");

        assert_eq!(claims.user_pubkey, expected);
    }

    #[test]
    fn test_cip8_verification_rejects_missing_user_pubkey() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: "{}".to_string(),
            signature: vec![0u8; 8],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );
        let err = verify_cip8_cose_signature(&request, &payload).unwrap_err();
        assert!(format!("{err:?}").contains("Missing user_pubkey"));
    }

    #[test]
    fn test_cip8_verification_rejects_invalid_user_pubkey_hex() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: "{\"user_pubkey\":\"not-hex\"}".to_string(),
            signature: vec![0u8; 8],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );
        let err = verify_cip8_cose_signature(&request, &payload).unwrap_err();
        assert!(format!("{err:?}").contains("Invalid user_pubkey hex"));
    }

    #[test]
    fn test_cip8_verification_rejects_wrong_user_pubkey_length() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: "{\"user_pubkey\":\"abcd\"}".to_string(),
            signature: vec![0u8; 8],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );
        let err = verify_cip8_cose_signature(&request, &payload).unwrap_err();
        assert!(format!("{err:?}").contains("user_pubkey must be 32 bytes"));
    }

    #[test]
    fn test_cip8_verification_rejects_invalid_cose_bytes() {
        let (_sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: vec![0u8; 3],
            nonce: 1,
            network: "preprod".to_string(),
        };

        let payload = build_canonical_payload(
            &request,
            &PublicKey(pk_bytes.try_into().unwrap()),
            "addr_test1...",
        );
        let err = verify_cip8_cose_signature(&request, &payload).unwrap_err();
        assert!(format!("{err:?}").contains("Invalid COSE_Sign1"));
    }

    #[test]
    fn test_cip8_verification_rejects_missing_alg() {
        let (sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: build_custom_cip8_signature(
                &sk,
                Some(b"payload"),
                None,
                None,
            ),
            nonce: 1,
            network: "preprod".to_string(),
        };

        let err = verify_cip8_cose_signature(&request, b"payload").unwrap_err();
        assert!(format!("{err:?}").contains("Missing alg"));
    }

    #[test]
    fn test_cip8_verification_rejects_unsupported_alg() {
        let (sk, pk_bytes) = gen_key();
        let payload = b"payload";
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: build_custom_cip8_signature(
                &sk,
                Some(payload),
                Some(coset::RegisteredLabelWithPrivate::Assigned(
                    iana::Algorithm::ES256,
                )),
                None,
            ),
            nonce: 1,
            network: "preprod".to_string(),
        };

        let err = verify_cip8_cose_signature(&request, payload).unwrap_err();
        assert!(format!("{err:?}").contains("Unsupported alg"));
    }

    #[test]
    fn test_cip8_verification_rejects_missing_cose_payload() {
        let (sk, pk_bytes) = gen_key();
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: build_custom_cip8_signature(
                &sk,
                None,
                Some(coset::RegisteredLabelWithPrivate::Assigned(
                    iana::Algorithm::EdDSA,
                )),
                None,
            ),
            nonce: 1,
            network: "preprod".to_string(),
        };

        let err = verify_cip8_cose_signature(&request, b"payload").unwrap_err();
        assert!(format!("{err:?}").contains("COSE payload missing"));
    }

    #[test]
    fn test_cip8_verification_rejects_wrong_signature_length() {
        let (sk, pk_bytes) = gen_key();
        let payload = b"payload";
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: build_custom_cip8_signature(
                &sk,
                Some(payload),
                Some(coset::RegisteredLabelWithPrivate::Assigned(
                    iana::Algorithm::EdDSA,
                )),
                Some(vec![7u8; 32]),
            ),
            nonce: 1,
            network: "preprod".to_string(),
        };

        let err = verify_cip8_cose_signature(&request, payload).unwrap_err();
        assert!(format!("{err:?}").contains("COSE signature must be 64 bytes"));
    }

    #[test]
    fn test_cip8_verification_rejects_invalid_signature() {
        let (sk, pk_bytes) = gen_key();
        let payload = b"payload";
        let mut request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![],
            message: format!(
                "{{\"user_pubkey\":\"{}\"}}",
                hex::encode(&pk_bytes)
            ),
            signature: build_cip8_signature(&sk, payload),
            nonce: 1,
            network: "preprod".to_string(),
        };

        let mut cose =
            CoseSign1::from_tagged_slice(&request.signature).unwrap();
        cose.signature[0] ^= 0xff;
        request.signature = cose.to_tagged_vec().unwrap();

        let err = verify_cip8_cose_signature(&request, payload).unwrap_err();
        assert!(format!("{err:?}").contains("verification failed"));
    }
}

/// Sign blinded outputs with delegate key.
///
/// Each output carries a blinded commitment point in its `signature` field
/// (a compressed Ristretto point). The node decompresses and signs the
/// point directly, allowing the client to unblind the result with the
/// corresponding blinding factor.
fn sign_outputs(
    request: &DepositRequest,
    keypair: &mugraph_core::types::Keypair,
) -> Result<Vec<BlindSignature>, Error> {
    let mut rng = rand::rng();
    let mut signatures = Vec::with_capacity(request.outputs.len());

    for commitment in &request.outputs {
        let blinded_point = commitment.signature.0.to_point()?;

        let blinded_sig =
            crypto::sign_blinded(&mut rng, &keypair.secret_key, &blinded_point);

        signatures.push(blinded_sig);
    }

    Ok(signatures)
}

#[cfg(test)]
mod wallet_tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::{config::Config, database::Database};

    fn test_context() -> Context {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("db.redb");
        let database = Arc::new(Database::setup(db_path).unwrap());
        database.migrate().unwrap();
        std::mem::forget(dir);

        let config = Config::Server {
            addr: "127.0.0.1:9999".parse().unwrap(),
            seed: Some(7),
            secret_key: None,
            cardano_network: "preprod".to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: Some("test".to_string()),
            cardano_provider_url: None,
            cardano_payment_sk: None,
            xnode_peer_registry_file: None,
            xnode_node_id: "node://local".to_string(),
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: Some(1_000_000),
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
            dev_mode: true,
        };
        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
            peer_registry: None,
        }
    }

    #[test]
    fn store_wallet_if_absent_keeps_existing_wallet() {
        let ctx = test_context();
        let first = mugraph_core::types::CardanoWallet::new(
            vec![1u8; 32],
            vec![2u8; 32],
            vec![3u8; 10],
            vec![4u8; 28],
            "addr_test1first".to_string(),
            "preprod".to_string(),
        );
        let second = mugraph_core::types::CardanoWallet::new(
            vec![9u8; 32],
            vec![8u8; 32],
            vec![7u8; 10],
            vec![6u8; 28],
            "addr_test1second".to_string(),
            "preprod".to_string(),
        );

        let saved = store_wallet_if_absent(&ctx, first.clone()).unwrap();
        assert_eq!(saved.script_address, first.script_address);

        let selected = store_wallet_if_absent(&ctx, second.clone()).unwrap();
        assert_eq!(selected.script_address, first.script_address);

        let read_tx = ctx.database.read().unwrap();
        let table = read_tx.open_table(CARDANO_WALLET).unwrap();
        let persisted = table.get("wallet").unwrap().unwrap().value();
        assert_eq!(persisted.script_address, first.script_address);
    }

    #[test]
    fn sign_outputs_produces_unblindable_signatures() {
        use mugraph_core::{
            crypto,
            types::{
                BlindSignature,
                Blinded,
                DleqProof,
                Hash,
                Note,
                Signature,
                UtxoReference,
            },
        };
        use rand::{SeedableRng, rngs::StdRng};

        let ctx = test_context();
        let mut rng = StdRng::seed_from_u64(42);

        // Build a note commitment the way a wallet would
        let note = Note {
            delegate: ctx.keypair.public_key,
            policy_id: Default::default(),
            asset_name: Default::default(),
            nonce: Hash::random(&mut rng),
            amount: 1000,
            signature: Signature::default(),
            dleq: None,
        };
        let commitment = note.commitment();

        // Client: blind the commitment
        let blinded = crypto::blind(&mut rng, commitment.as_ref());
        let compressed_point = Signature::from(blinded.point);

        // Pack the blinded point into a BlindSignature to carry in the request
        let output = BlindSignature {
            signature: Blinded(compressed_point),
            proof: DleqProof::default(),
        };

        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "00".repeat(32),
                index: 0,
            },
            outputs: vec![output],
            message: String::new(),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        };

        // Server: sign the outputs
        let signatures =
            sign_outputs(&request, &ctx.keypair).expect("sign must succeed");
        assert_eq!(signatures.len(), 1);

        let sig = &signatures[0];

        // Client: unblind the signature
        let unblinded = crypto::unblind_signature(
            &sig.signature,
            &blinded.factor,
            &ctx.keypair.public_key,
        )
        .expect("unblind must succeed");

        // Client: verify the unblinded signature against the commitment
        assert!(
            crypto::verify(
                &ctx.keypair.public_key,
                commitment.as_ref(),
                unblinded,
            )
            .expect("verify must not error"),
            "unblinded signature must verify",
        );
    }

    #[test]
    fn insert_deposit_if_absent_rejects_duplicates() {
        let ctx = test_context();
        let write_tx = ctx.database.write().unwrap();
        {
            let mut table = write_tx.open_table(DEPOSITS).unwrap();
            let utxo = UtxoRef::new([1u8; 32], 0);
            let record = mugraph_core::types::DepositRecord::new(1, 1, 100);

            insert_deposit_if_absent(&mut table, utxo.clone(), record.clone())
                .unwrap();
            let err =
                insert_deposit_if_absent(&mut table, utxo, record).unwrap_err();
            assert!(matches!(err, Error::InvalidInput { .. }));
        }
        write_tx.commit().unwrap();
    }
}

#[cfg(test)]
mod datum_tests {
    use ed25519_dalek::SigningKey;
    use pallas_codec::minicbor;
    use pallas_primitives::{
        BoundedBytes,
        Constr,
        MaybeIndefArray,
        alonzo::PlutusData,
    };

    use super::*;
    use crate::provider::{AssetAmount, UtxoInfo};

    fn mk_request_with_user_pubkey(user_pk: &[u8; 32]) -> DepositRequest {
        DepositRequest {
            utxo: mugraph_core::types::UtxoReference {
                tx_hash: "ab".repeat(32),
                index: 0,
            },
            outputs: vec![BlindSignature::default()],
            message: format!(r#"{{"user_pubkey":"{}"}}"#, hex::encode(user_pk)),
            signature: vec![0u8; 64],
            nonce: 1,
            network: "preprod".to_string(),
        }
    }

    fn mk_wallet(payment_vk: &[u8; 32]) -> mugraph_core::types::CardanoWallet {
        mugraph_core::types::CardanoWallet::new(
            vec![9u8; 32],
            payment_vk.to_vec(),
            vec![],
            vec![],
            "addr_test1datumcheck".to_string(),
            "preprod".to_string(),
        )
    }

    fn mk_utxo_with_datum_hex(datum_hex: Option<String>) -> UtxoInfo {
        UtxoInfo {
            tx_hash: "ab".repeat(32),
            output_index: 0,
            address: "addr_test1datumcheck".to_string(),
            amount: vec![AssetAmount {
                unit: "lovelace".to_string(),
                quantity: "1000000".to_string(),
            }],
            datum_hash: None,
            datum: datum_hex,
            script_ref: None,
            block_height: Some(100),
        }
    }

    fn mk_datum_hex(tag: u64, fields: Vec<Vec<u8>>) -> String {
        let datum = PlutusData::Constr(Constr {
            tag,
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

    fn mk_valid_datum_hex(
        request: &DepositRequest,
        wallet: &mugraph_core::types::CardanoWallet,
        delegate_pk: &PublicKey,
        user_pk: &[u8; 32],
    ) -> String {
        let user_hash = csl::PublicKey::from_bytes(user_pk)
            .expect("valid user key")
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
            .expect("valid node key")
            .hash()
            .to_bytes();
        let intent_hash =
            compute_intent_hash(request, delegate_pk, &wallet.script_address);

        mk_datum_hex(121, vec![user_hash, node_hash, intent_hash.to_vec()])
    }

    #[test]
    fn validate_deposit_datum_rejects_missing_datum() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);
        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let utxo = mk_utxo_with_datum_hex(None);
        let delegate_pk = PublicKey([3u8; 32]);

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(format!("{err:?}").contains("missing inline datum"));
    }

    #[test]
    fn validate_deposit_datum_rejects_non_cbor_datum() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);
        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let utxo = mk_utxo_with_datum_hex(Some("00ff".to_string()));
        let delegate_pk = PublicKey([3u8; 32]);

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn validate_deposit_datum_rejects_user_hash_mismatch() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);
        let wrong_user_sk = SigningKey::from_bytes(&[4u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();
        let wrong_user_pk = wrong_user_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let datum_hex =
            mk_valid_datum_hex(&request, &wallet, &delegate_pk, &wrong_user_pk);
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("Datum user_pubkey_hash does not match")
        );
    }

    #[test]
    fn validate_deposit_datum_accepts_valid_payload_binding() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let datum_hex =
            mk_valid_datum_hex(&request, &wallet, &delegate_pk, &user_pk);
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
            .expect("valid datum must pass");
    }

    #[test]
    fn validate_deposit_datum_rejects_wrong_constructor() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let user_hash = csl::PublicKey::from_bytes(&user_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
            .unwrap()
            .hash()
            .to_bytes();
        let intent_hash =
            compute_intent_hash(&request, &delegate_pk, &wallet.script_address);

        let datum_hex =
            mk_datum_hex(122, vec![user_hash, node_hash, intent_hash.to_vec()]);
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(format!("{err:?}").contains("Unexpected datum constructor"));
    }

    #[test]
    fn validate_deposit_datum_rejects_wrong_field_count() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let user_hash = csl::PublicKey::from_bytes(&user_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
            .unwrap()
            .hash()
            .to_bytes();

        let datum_hex = mk_datum_hex(121, vec![user_hash, node_hash]);
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(format!("{err:?}").contains("expected 3"));
    }

    #[test]
    fn validate_deposit_datum_rejects_node_hash_mismatch() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);
        let wrong_node_sk = SigningKey::from_bytes(&[8u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();
        let wrong_node_pk = wrong_node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let user_hash = csl::PublicKey::from_bytes(&user_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let wrong_node_hash = csl::PublicKey::from_bytes(&wrong_node_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let intent_hash =
            compute_intent_hash(&request, &delegate_pk, &wallet.script_address);

        let datum_hex = mk_datum_hex(
            121,
            vec![user_hash, wrong_node_hash, intent_hash.to_vec()],
        );
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("node_pubkey_hash does not match this node")
        );
    }

    #[test]
    fn validate_deposit_datum_rejects_intent_hash_mismatch() {
        let user_sk = SigningKey::from_bytes(&[1u8; 32]);
        let node_sk = SigningKey::from_bytes(&[2u8; 32]);

        let user_pk = user_sk.verifying_key().to_bytes();
        let node_pk = node_sk.verifying_key().to_bytes();

        let request = mk_request_with_user_pubkey(&user_pk);
        let wallet = mk_wallet(&node_pk);
        let delegate_pk = PublicKey([3u8; 32]);

        let user_hash = csl::PublicKey::from_bytes(&user_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
            .unwrap()
            .hash()
            .to_bytes();
        let wrong_intent = vec![9u8; 32];

        let datum_hex =
            mk_datum_hex(121, vec![user_hash, node_hash, wrong_intent]);
        let utxo = mk_utxo_with_datum_hex(Some(datum_hex));

        let err =
            validate_deposit_datum(&request, &wallet, &utxo, &delegate_pk)
                .unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("intent_hash does not match canonical payload")
        );
    }
}

#[cfg(test)]
mod amount_validation_tests {
    use super::*;
    use crate::provider::{AssetAmount, UtxoInfo};

    fn request_with_output_count(count: usize) -> DepositRequest {
        DepositRequest {
            utxo: mugraph_core::types::UtxoReference {
                tx_hash: "ab".repeat(32),
                index: 0,
            },
            outputs: vec![BlindSignature::default(); count],
            message: "{}".to_string(),
            signature: vec![],
            nonce: 1,
            network: "preprod".to_string(),
        }
    }

    fn utxo_with_amounts(amount: Vec<AssetAmount>) -> UtxoInfo {
        UtxoInfo {
            tx_hash: "ab".repeat(32),
            output_index: 0,
            address: "addr_test1amountcheck".to_string(),
            amount,
            datum_hash: None,
            datum: None,
            script_ref: None,
            block_height: Some(100),
        }
    }

    #[test]
    fn validate_deposit_amounts_rejects_missing_outputs() {
        let request = request_with_output_count(0);
        let utxo = utxo_with_amounts(vec![AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "1000000".to_string(),
        }]);

        let err =
            validate_deposit_amounts(&request, &utxo, 1_000_000).unwrap_err();
        assert!(format!("{err:?}").contains("No outputs provided for deposit"));
    }

    #[test]
    fn validate_deposit_amounts_rejects_too_few_outputs_for_distinct_assets() {
        let request = request_with_output_count(1);
        let utxo = utxo_with_amounts(vec![
            AssetAmount {
                unit: "lovelace".to_string(),
                quantity: "1000000".to_string(),
            },
            AssetAmount {
                unit: format!("{}{}", "11".repeat(28), "746f6b656e"),
                quantity: "1".to_string(),
            },
        ]);

        let err =
            validate_deposit_amounts(&request, &utxo, 1_000_000).unwrap_err();
        assert!(format!("{err:?}").contains("Insufficient outputs"));
    }

    #[test]
    fn validate_deposit_amounts_rejects_more_outputs_than_total_units() {
        let request = request_with_output_count(3);
        let utxo = utxo_with_amounts(vec![AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "2".to_string(),
        }]);

        let err = validate_deposit_amounts(&request, &utxo, 1).unwrap_err();
        assert!(format!("{err:?}").contains("Too many outputs"));
    }

    #[test]
    fn validate_deposit_amounts_rejects_below_minimum_lovelace() {
        let request = request_with_output_count(1);
        let utxo = utxo_with_amounts(vec![AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "999999".to_string(),
        }]);

        let err =
            validate_deposit_amounts(&request, &utxo, 1_000_000).unwrap_err();
        assert!(format!("{err:?}").contains("below minimum"));
    }

    #[test]
    fn validate_deposit_amounts_rejects_invalid_asset_quantity() {
        let request = request_with_output_count(1);
        let utxo = utxo_with_amounts(vec![AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "not-a-number".to_string(),
        }]);

        let err =
            validate_deposit_amounts(&request, &utxo, 1_000_000).unwrap_err();
        assert!(format!("{err:?}").contains("Invalid asset quantity"));
    }

    #[test]
    fn validate_deposit_amounts_accepts_exact_minimum_and_unique_asset_boundary()
     {
        let request = request_with_output_count(2);
        let utxo = utxo_with_amounts(vec![
            AssetAmount {
                unit: "lovelace".to_string(),
                quantity: "1000000".to_string(),
            },
            AssetAmount {
                unit: format!("{}{}", "22".repeat(28), "746f6b656e"),
                quantity: "1".to_string(),
            },
        ]);

        validate_deposit_amounts(&request, &utxo, 1_000_000)
            .expect("exact minimum and asset-count boundary should pass");
    }
}

#[cfg(test)]
mod handle_deposit_flow_tests {
    use std::sync::Arc;

    use axum::{
        Router,
        extract::Path,
        http::StatusCode,
        response::IntoResponse,
        routing::get,
    };
    use coset::{
        CoseSign1,
        CoseSign1Builder,
        Header,
        ProtectedHeader,
        TaggedCborSerializable,
        iana,
    };
    use ed25519_dalek::{Signer, SigningKey};
    use pallas_codec::minicbor;
    use pallas_primitives::{
        BoundedBytes,
        Constr,
        MaybeIndefArray,
        alonzo::PlutusData,
    };
    use serde_json::json;

    use super::*;
    use crate::{config::Config, database::Database};

    fn build_cip8_signature(sk: &SigningKey, payload: &[u8]) -> Vec<u8> {
        let header = Header {
            alg: Some(coset::RegisteredLabelWithPrivate::Assigned(
                iana::Algorithm::EdDSA,
            )),
            ..Default::default()
        };
        let tbs = CoseSign1 {
            protected: ProtectedHeader {
                original_data: None,
                header: header.clone(),
            },
            unprotected: Header::default(),
            payload: Some(payload.to_vec()),
            signature: vec![],
        }
        .tbs_data(&[]);
        let sig = sk.sign(&tbs);

        CoseSign1Builder::new()
            .protected(header)
            .payload(payload.to_vec())
            .signature(sig.to_vec())
            .build()
            .to_tagged_vec()
            .unwrap()
    }

    fn mk_context(provider_url: String) -> Context {
        let db_path = std::env::temp_dir().join(format!(
            "mugraph-handle-deposit-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let database = Arc::new(Database::setup(db_path).unwrap());
        database.migrate().unwrap();

        let config = Config::Server {
            addr: "127.0.0.1:9999".parse().unwrap(),
            seed: Some(42),
            secret_key: None,
            cardano_network: "preprod".to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: Some("test".to_string()),
            cardano_provider_url: Some(provider_url),
            cardano_payment_sk: None,
            xnode_peer_registry_file: None,
            xnode_node_id: "node://local".to_string(),
            deposit_confirm_depth: 5,
            deposit_expiration_blocks: 1440,
            min_deposit_value: Some(1_000_000),
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
            dev_mode: true,
        };

        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
            peer_registry: None,
        }
    }

    fn insert_wallet(ctx: &Context, payment_vk: Vec<u8>, script_address: &str) {
        let w = ctx.database.write().unwrap();
        {
            let mut t = w.open_table(CARDANO_WALLET).unwrap();
            t.insert(
                "wallet",
                &mugraph_core::types::CardanoWallet::new(
                    vec![1u8; 32],
                    payment_vk,
                    vec![],
                    vec![],
                    script_address.to_string(),
                    "preprod".to_string(),
                ),
            )
            .unwrap();
        }
        w.commit().unwrap();
    }

    fn mk_datum_cbor_hex(
        user_hash: Vec<u8>,
        node_hash: Vec<u8>,
        intent_hash: Vec<u8>,
    ) -> String {
        let datum = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Def(vec![
                PlutusData::BoundedBytes(BoundedBytes::from(user_hash)),
                PlutusData::BoundedBytes(BoundedBytes::from(node_hash)),
                PlutusData::BoundedBytes(BoundedBytes::from(intent_hash)),
            ]),
        });

        hex::encode(minicbor::to_vec(&datum).unwrap())
    }

    async fn spawn_provider_mock_with_outputs(
        script_address: String,
        datum_cbor_hex: String,
        tip_height: u64,
        include_output: bool,
    ) -> String {
        async fn tx_info() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"block_height": 90})))
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<(
                String,
                String,
                bool,
            )>,
        ) -> impl IntoResponse {
            let (script_address, _datum_hex, include_output) = state;

            let outputs = if include_output {
                vec![json!({
                    "output_index": 0,
                    "address": script_address,
                    "amount": [{"unit":"lovelace","quantity":"1000000"}],
                    "data_hash": "datumhash",
                    "reference_script_hash": null
                })]
            } else {
                Vec::new()
            };

            (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": tx_hash,
                    "outputs": outputs
                })),
            )
        }

        async fn datum_cbor(
            axum::extract::State(state): axum::extract::State<(
                String,
                String,
                bool,
            )>,
        ) -> impl IntoResponse {
            let (_script_address, datum_hex, _include_output) = state;
            (StatusCode::OK, axum::Json(json!({"cbor": datum_hex})))
        }

        let app = Router::new()
            .route("/blocks/latest", get(move || async move {
                (StatusCode::OK, axum::Json(json!({"slot": 1000, "hash": "tip", "height": tip_height})))
            }))
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .route("/scripts/datum/{datum_hash}/cbor", get(datum_cbor))
            .with_state((script_address, datum_cbor_hex, include_output));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    async fn spawn_provider_mock(
        script_address: String,
        datum_cbor_hex: String,
        tip_height: u64,
    ) -> String {
        spawn_provider_mock_with_outputs(
            script_address,
            datum_cbor_hex,
            tip_height,
            true,
        )
        .await
    }

    async fn spawn_provider_mock_without_block_height(
        script_address: String,
        datum_cbor_hex: String,
        tip_height: u64,
    ) -> String {
        async fn tx_info() -> impl IntoResponse {
            StatusCode::NOT_FOUND
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<(String, String)>,
        ) -> impl IntoResponse {
            let (script_address, _datum_hex) = state;
            (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": tx_hash,
                    "outputs": [{
                        "output_index": 0,
                        "address": script_address,
                        "amount": [{"unit":"lovelace","quantity":"1000000"}],
                        "data_hash": "datumhash",
                        "reference_script_hash": null
                    }]
                })),
            )
        }

        async fn datum_cbor(
            axum::extract::State(state): axum::extract::State<(String, String)>,
        ) -> impl IntoResponse {
            let (_script_address, datum_hex) = state;
            (StatusCode::OK, axum::Json(json!({"cbor": datum_hex})))
        }

        let app = Router::new()
            .route("/blocks/latest", get(move || async move {
                (StatusCode::OK, axum::Json(json!({"slot": 1000, "hash": "tip", "height": tip_height})))
            }))
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .route("/scripts/datum/{datum_hash}/cbor", get(datum_cbor))
            .with_state((script_address, datum_cbor_hex));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    async fn spawn_provider_mock_with_tip_failure(
        script_address: String,
        datum_cbor_hex: String,
    ) -> String {
        async fn tip() -> impl IntoResponse {
            (StatusCode::INTERNAL_SERVER_ERROR, "tip failed")
        }

        async fn tx_info() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"block_height": 90})))
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<(String, String)>,
        ) -> impl IntoResponse {
            let (script_address, _datum_hex) = state;
            (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": tx_hash,
                    "outputs": [{
                        "output_index": 0,
                        "address": script_address,
                        "amount": [{"unit":"lovelace","quantity":"1000000"}],
                        "data_hash": "datumhash",
                        "reference_script_hash": null
                    }]
                })),
            )
        }

        async fn datum_cbor(
            axum::extract::State(state): axum::extract::State<(String, String)>,
        ) -> impl IntoResponse {
            let (_script_address, datum_hex) = state;
            (StatusCode::OK, axum::Json(json!({"cbor": datum_hex})))
        }

        let app = Router::new()
            .route("/blocks/latest", get(tip))
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .route("/scripts/datum/{datum_hash}/cbor", get(datum_cbor))
            .with_state((script_address, datum_cbor_hex));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    fn prepare_request_and_datum(
        ctx: &Context,
        user_sk: &SigningKey,
        node_pk: &[u8; 32],
    ) -> (DepositRequest, String) {
        let user_pk = user_sk.verifying_key().to_bytes();

        let mut request = DepositRequest {
            utxo: mugraph_core::types::UtxoReference {
                tx_hash: "ab".repeat(32),
                index: 0,
            },
            outputs: vec![BlindSignature::default()],
            message: format!(r#"{{"user_pubkey":"{}"}}"#, hex::encode(user_pk)),
            signature: vec![],
            nonce: 7,
            network: "preprod".to_string(),
        };

        insert_wallet(ctx, node_pk.to_vec(), "addr_test1script");
        let wallet = {
            let r = ctx.database.read().unwrap();
            let t = r.open_table(CARDANO_WALLET).unwrap();
            t.get("wallet").unwrap().unwrap().value()
        };

        let intent = compute_intent_hash(
            &request,
            &ctx.keypair.public_key,
            &wallet.script_address,
        );
        let user_hash = csl::PublicKey::from_bytes(&user_pk)
            .unwrap()
            .hash()
            .to_bytes();
        let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
            .unwrap()
            .hash()
            .to_bytes();
        let datum_cbor_hex =
            mk_datum_cbor_hex(user_hash, node_hash, intent.to_vec());

        let payload = build_canonical_payload(
            &request,
            &ctx.keypair.public_key,
            "addr_test1script",
        );
        request.signature = build_cip8_signature(user_sk, &payload);

        (request, datum_cbor_hex)
    }

    #[tokio::test]
    async fn handle_deposit_happy_path_records_deposit() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let response = handle_deposit(&request, &ctx)
            .await
            .expect("deposit accepted");
        assert!(matches!(response, Response::Deposit { .. }));

        let r = ctx.database.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        assert!(deposits.get(&utxo_ref).unwrap().is_some());
    }

    #[tokio::test]
    async fn handle_deposit_happy_path_persists_expected_intent_hash() {
        let user_sk = SigningKey::from_bytes(&[14u8; 32]);
        let node_sk = SigningKey::from_bytes(&[25u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        handle_deposit(&request, &ctx)
            .await
            .expect("deposit accepted");

        let r = ctx.database.read().unwrap();
        let wallets = r.open_table(CARDANO_WALLET).unwrap();
        let wallet = wallets.get("wallet").unwrap().unwrap().value();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        let record = deposits.get(&utxo_ref).unwrap().unwrap().value();

        let expected_intent_hash = compute_intent_hash(
            &request,
            &ctx.keypair.public_key,
            &wallet.script_address,
        );

        assert_eq!(record.intent_hash, expected_intent_hash);
    }

    #[tokio::test]
    async fn invalid_deposit_message_does_not_record_a_deposit() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (mut request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);
        request.message = "{}".to_string();

        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Missing user_pubkey"));

        let r = ctx.database.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        assert!(deposits.get(&utxo_ref).unwrap().is_none());
    }

    #[tokio::test]
    async fn handle_deposit_rejects_insufficient_confirmations() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        // tx block is 90 in mock; tip 92 => 2 confirmations < configured depth 5.
        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            92,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("not sufficiently confirmed"));

        let r = ctx.database.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        assert!(deposits.get(&utxo_ref).unwrap().is_none());
    }

    #[tokio::test]
    async fn handle_deposit_accepts_exact_confirmation_threshold() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        // tx block is 90 in mock; tip 95 => exactly 5 confirmations, which should pass.
        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            95,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let response = handle_deposit(&request, &ctx)
            .await
            .expect("deposit accepted at confirmation threshold");
        assert!(matches!(response, Response::Deposit { .. }));
    }

    #[tokio::test]
    async fn handle_deposit_rejects_wrong_script_address() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock(
            "addr_test1different".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("not at script address"));
    }

    #[tokio::test]
    async fn handle_deposit_rejects_missing_utxo() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock_with_outputs(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
            false,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("UTxO not found on chain"));
    }

    #[tokio::test]
    async fn handle_deposit_rejects_missing_block_height_without_recording_deposit()
     {
        let user_sk = SigningKey::from_bytes(&[12u8; 32]);
        let node_sk = SigningKey::from_bytes(&[23u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock_without_block_height(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("Cannot verify UTxO confirmation depth")
        );

        let r = ctx.database.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        assert!(deposits.get(&utxo_ref).unwrap().is_none());
    }

    #[tokio::test]
    async fn handle_deposit_surfaces_tip_lookup_failures_without_recording_deposit()
     {
        let user_sk = SigningKey::from_bytes(&[13u8; 32]);
        let node_sk = SigningKey::from_bytes(&[24u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock_with_tip_failure(
            "addr_test1script".to_string(),
            datum_cbor_hex,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("Failed to get chain tip for confirm depth check")
        );

        let r = ctx.database.read().unwrap();
        let deposits = r.open_table(DEPOSITS).unwrap();
        let utxo_ref = UtxoRef::new([0xabu8; 32], 0);
        assert!(deposits.get(&utxo_ref).unwrap().is_none());
    }

    #[tokio::test]
    async fn handle_deposit_rejects_provider_datum_mismatch() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, _datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        // Provider serves malformed/incorrect datum payload.
        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            "00".to_string(),
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[tokio::test]
    async fn handle_deposit_rejects_duplicate_deposit() {
        let user_sk = SigningKey::from_bytes(&[11u8; 32]);
        let node_sk = SigningKey::from_bytes(&[22u8; 32]);
        let node_pk = node_sk.verifying_key().to_bytes();

        let seed_ctx = mk_context("http://127.0.0.1:1".to_string());
        let (request, datum_cbor_hex) =
            prepare_request_and_datum(&seed_ctx, &user_sk, &node_pk);

        let url = spawn_provider_mock(
            "addr_test1script".to_string(),
            datum_cbor_hex,
            100,
        )
        .await;
        let ctx = mk_context(url);
        insert_wallet(&ctx, node_pk.to_vec(), "addr_test1script");

        handle_deposit(&request, &ctx)
            .await
            .expect("first deposit accepted");
        let err = handle_deposit(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("already processed"));
    }
}
