//! Note operations that go through the delegate.

use mugraph_core::{
    builder::RefreshBuilder,
    keyset,
    types::{
        Asset, BlindSignature, BlindedOutput, Note, PendingNote, PublicKey,
        blind_new_note,
    },
};

use crate::{
    node_client::{NodeClient, NodeClientError},
    store::Store,
};

/// Gets the delegate's denomination keys for `asset`.
pub async fn fetch_keyset(
    client: &NodeClient,
    asset: &Asset,
) -> Result<Vec<PublicKey>, String> {
    client.keys(asset).await.map_err(|e| e.to_string())
}

/// Checks a note that another user sent. The note must carry its DLEQ
/// proof and blinding factor, and the proof must be valid for the
/// delegate's key for the asset and amount of the note.
pub fn verify_received_note(note: &Note, keys: &[PublicKey]) -> bool {
    let Some(proof) = note.dleq.as_ref() else {
        return false;
    };
    let Ok(public_key) = keyset::keyset_public_key(keys, note.amount) else {
        return false;
    };

    mugraph_core::crypto::verify_note_proof(
        &public_key,
        note.commitment().as_ref(),
        note.signature,
        proof,
    )
    .unwrap_or(false)
}

/// Swaps `inputs` for new notes through the delegate. Each output amount
/// becomes one note for each of its denominations. The wallet saves each
/// blinding factor before it sends the request, so it can recover the
/// notes if the response does not arrive.
///
/// Returns the new notes in output order. The caller marks the inputs as
/// spent and stores the new notes.
pub async fn refresh_through_node(
    store: &Store,
    network: &str,
    client: &NodeClient,
    inputs: &[Note],
    outputs: &[(Asset, u64)],
) -> Result<Vec<Note>, String> {
    let mut builder = RefreshBuilder::new();
    for note in inputs {
        builder = builder.input(note.clone());
    }
    for (asset, amount) in outputs {
        builder = builder.output(asset.policy_id, asset.asset_name, *amount);
    }
    let mut refresh = builder.build().map_err(|e| e.to_string())?;

    let secrets = refresh.blind_outputs(&mut rand::rng());
    for secret in &secrets {
        store
            .put_blinding_factor(
                network,
                &secret.nonce,
                &secret.blinding_factor.0,
            )
            .map_err(|e| e.to_string())?;
    }

    let signatures = match client.refresh(&refresh).await {
        Ok(signatures) => signatures,
        Err(e) => {
            // The node refused the request, so it signed nothing and the
            // blinding factors have no use. After a network error, the
            // node can have signed the outputs, so keep them.
            if matches!(e, NodeClientError::Node { .. }) {
                for secret in &secrets {
                    let _ =
                        store.delete_blinding_factor(network, &secret.nonce);
                }
            }
            return Err(e.to_string());
        }
    };
    if signatures.len() != secrets.len() {
        return Err(format!(
            "node returned {} signatures for {} outputs",
            signatures.len(),
            secrets.len()
        ));
    }

    let mut keysets: std::collections::HashMap<Asset, Vec<PublicKey>> =
        std::collections::HashMap::new();
    let mut notes = Vec::with_capacity(secrets.len());
    for (secret, signature) in secrets.iter().zip(&signatures) {
        let atom = &refresh.atoms[secret.atom_index];
        let asset = refresh.asset_ids[atom.asset_id as usize];
        if let std::collections::hash_map::Entry::Vacant(entry) =
            keysets.entry(asset)
        {
            entry.insert(fetch_keyset(client, &asset).await?);
        }
        let public_key =
            keyset::keyset_public_key(&keysets[&asset], atom.amount)
                .map_err(|e| e.to_string())?;

        let note = refresh
            .unblind_output(secret, signature, &public_key)
            .map_err(|e| e.to_string())?;
        notes.push(note);
    }

    Ok(notes)
}

/// Makes and blinds new notes for `outputs`. Each amount becomes one note
/// for each of its denominations. Saves each blinding factor before the
/// caller sends the outputs to the node.
pub fn blind_new_notes(
    store: &Store,
    network: &str,
    delegate: PublicKey,
    outputs: &[(Asset, u64)],
) -> Result<(Vec<BlindedOutput>, Vec<PendingNote>), String> {
    let mut rng = rand::rng();
    let mut blinded = Vec::new();
    let mut pending = Vec::new();

    for (asset, amount) in outputs {
        for part in keyset::split_amount(*amount) {
            let (output, note) =
                blind_new_note(&mut rng, delegate, asset, part);
            store
                .put_blinding_factor(
                    network,
                    &note.note.nonce,
                    &note.blinding_factor.0,
                )
                .map_err(|e| e.to_string())?;
            blinded.push(output);
            pending.push(note);
        }
    }

    Ok((blinded, pending))
}

/// Unblinds the node's signatures for `pending` and checks each one with
/// the delegate's key for its asset and amount.
pub async fn finish_notes(
    client: &NodeClient,
    pending: Vec<PendingNote>,
    signatures: &[BlindSignature],
) -> Result<Vec<Note>, String> {
    if signatures.len() != pending.len() {
        return Err(format!(
            "node returned {} signatures for {} outputs",
            signatures.len(),
            pending.len()
        ));
    }

    let mut keysets: std::collections::HashMap<Asset, Vec<PublicKey>> =
        std::collections::HashMap::new();
    let mut notes = Vec::with_capacity(pending.len());
    for (note, signature) in pending.into_iter().zip(signatures) {
        let asset = Asset {
            policy_id: note.note.policy_id,
            asset_name: note.note.asset_name,
        };
        if let std::collections::hash_map::Entry::Vacant(entry) =
            keysets.entry(asset)
        {
            entry.insert(fetch_keyset(client, &asset).await?);
        }
        let public_key =
            keyset::keyset_public_key(&keysets[&asset], note.note.amount)
                .map_err(|e| e.to_string())?;

        notes.push(
            note.finish(signature, &public_key)
                .map_err(|e| e.to_string())?,
        );
    }

    Ok(notes)
}

#[cfg(test)]
pub(crate) mod testing {
    use std::sync::Arc;

    use axum::{Json, Router, extract::State, routing::post};
    use mugraph_core::types::{Keypair, Request, Response};
    use mugraph_node::database::Database;
    use tempfile::TempDir;

    use crate::node_client::NodeClient;

    #[derive(Clone)]
    struct TestNode {
        keypair: Keypair,
        database: Arc<Database>,
    }

    async fn rpc(
        State(node): State<TestNode>,
        Json(request): Json<Request>,
    ) -> Json<Response> {
        let response = match request {
            Request::Refresh(refresh) => mugraph_node::routes::refresh(
                &refresh,
                node.keypair,
                &node.database,
            )
            .unwrap_or_else(|e| Response::Error {
                reason: e.to_string(),
            }),
            Request::Keys {
                policy_id,
                asset_name,
            } => Response::Keys {
                keys: mugraph_core::keyset::keyset(
                    &node.keypair.secret_key,
                    &mugraph_core::types::Asset {
                        policy_id,
                        asset_name,
                    },
                ),
            },
            other => Response::Error {
                reason: format!("test node does not handle {other:?}"),
            },
        };
        Json(response)
    }

    /// Starts a node on a local port that runs the real refresh code.
    pub async fn spawn_node(keypair: Keypair) -> (NodeClient, TempDir) {
        let dir = TempDir::new().unwrap();
        let database =
            Arc::new(Database::setup(dir.path().join("node.redb")).unwrap());
        database.migrate().unwrap();

        let app = Router::new()
            .route("/rpc", post(rpc))
            .with_state(TestNode { keypair, database });
        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = reqwest::Url::parse(&format!("http://{addr}")).unwrap();
        (NodeClient::new(&url).unwrap(), dir)
    }
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::Keypair;
    use rand::{SeedableRng, rngs::StdRng};
    use tempfile::TempDir;

    use super::*;

    fn store() -> (Store, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = Store::open(&dir.path().join("wallet.redb")).unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn refresh_through_node_returns_notes_that_verify() {
        let mut rng = StdRng::seed_from_u64(1);
        let keypair = Keypair::random(&mut rng);
        let asset = Asset::default();
        let note = keyset::issue_note(&mut rng, &keypair.secret_key, &asset, 8)
            .unwrap();
        let (client, _node_dir) = testing::spawn_node(keypair).await;
        let (store, _dir) = store();

        let notes = refresh_through_node(
            &store,
            "preprod",
            &client,
            &[note],
            &[(asset, 6), (asset, 2)],
        )
        .await
        .unwrap();

        let keys = fetch_keyset(&client, &asset).await.unwrap();
        let amounts: Vec<u64> = notes.iter().map(|n| n.amount).collect();
        assert_eq!(amounts, vec![2, 4, 2]);
        for note in &notes {
            assert!(verify_received_note(note, &keys));
            assert!(
                store
                    .get_blinding_factor("preprod", &note.nonce)
                    .unwrap()
                    .is_some(),
                "blinding factor must be saved before the request"
            );
        }
    }

    #[tokio::test]
    async fn refresh_through_node_fails_when_the_node_rejects() {
        let mut rng = StdRng::seed_from_u64(2);
        let keypair = Keypair::random(&mut rng);
        let asset = Asset::default();
        let note = keyset::issue_note(&mut rng, &keypair.secret_key, &asset, 8)
            .unwrap();
        let (client, _node_dir) = testing::spawn_node(keypair).await;
        let (store, _dir) = store();

        let first = refresh_through_node(
            &store,
            "preprod",
            &client,
            std::slice::from_ref(&note),
            &[(asset, 8)],
        )
        .await
        .unwrap();
        for new_note in &first {
            store
                .finalize_note(
                    "preprod",
                    new_note,
                    crate::store::NoteStatus::Available,
                    0,
                )
                .unwrap();
        }
        let second = refresh_through_node(
            &store,
            "preprod",
            &client,
            &[note],
            &[(asset, 8)],
        )
        .await;

        assert!(second.is_err(), "a spent note must not refresh again");
        assert!(
            store
                .scan_orphaned_blinding_factors("preprod")
                .unwrap()
                .is_empty(),
            "a rejected request must not leave blinding factors to recover"
        );
    }

    #[test]
    fn verify_received_note_needs_a_valid_proof() {
        let mut rng = StdRng::seed_from_u64(3);
        let keypair = Keypair::random(&mut rng);
        let asset = Asset::default();
        let keys = keyset::keyset(&keypair.secret_key, &asset);
        let note = keyset::issue_note(&mut rng, &keypair.secret_key, &asset, 8)
            .unwrap();

        assert!(verify_received_note(&note, &keys));

        let mut no_proof = note.clone();
        no_proof.dleq = None;
        assert!(!verify_received_note(&no_proof, &keys));

        let mut other_amount = note.clone();
        other_amount.amount = 16;
        assert!(!verify_received_note(&other_amount, &keys));

        let mut not_a_denomination = note;
        not_a_denomination.amount = 7;
        assert!(!verify_received_note(&not_a_denomination, &keys));
    }

    #[tokio::test]
    async fn blind_new_notes_then_finish_notes_makes_valid_notes() {
        let mut rng = StdRng::seed_from_u64(4);
        let keypair = Keypair::random(&mut rng);
        let asset = Asset::default();
        let (client, _node_dir) = testing::spawn_node(keypair).await;
        let (store, _dir) = store();

        let (outputs, pending) = blind_new_notes(
            &store,
            "preprod",
            keypair.public_key,
            &[(asset, 5)],
        )
        .unwrap();

        let amounts: Vec<u64> = outputs.iter().map(|o| o.amount).collect();
        assert_eq!(amounts, vec![1, 4]);
        for p in &pending {
            assert!(
                store
                    .get_blinding_factor("preprod", &p.note.nonce)
                    .unwrap()
                    .is_some()
            );
        }

        // The node signs each output with its denomination key.
        let signatures: Vec<BlindSignature> = outputs
            .iter()
            .map(|o| {
                keyset::sign_blinded(
                    &mut rng,
                    &keypair.secret_key,
                    &o.asset(),
                    o.amount,
                    &o.point.to_point().unwrap(),
                )
                .unwrap()
            })
            .collect();

        let notes = finish_notes(&client, pending, &signatures).await.unwrap();
        let keys = fetch_keyset(&client, &asset).await.unwrap();
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().all(|n| verify_received_note(n, &keys)));
    }

    #[tokio::test]
    async fn finish_notes_rejects_a_missing_signature() {
        let mut rng = StdRng::seed_from_u64(5);
        let keypair = Keypair::random(&mut rng);
        let (client, _node_dir) = testing::spawn_node(keypair).await;
        let (store, _dir) = store();

        let (_, pending) = blind_new_notes(
            &store,
            "preprod",
            keypair.public_key,
            &[(Asset::default(), 3)],
        )
        .unwrap();

        assert!(finish_notes(&client, pending, &[]).await.is_err());
    }
}
