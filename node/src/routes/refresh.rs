use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    keyset,
    types::{
        Asset, AssetName, Hash, Keypair, Note, PolicyId, Refresh, Response,
        Signature,
    },
};
use rand::{CryptoRng, RngCore};
use redb::ReadableTable;

use crate::database::{Database, NOTES};

#[inline]
pub fn emit_note<R: RngCore + CryptoRng>(
    keypair: &Keypair,
    policy_id: PolicyId,
    asset_name: AssetName,
    amount: u64,
    rng: &mut R,
) -> Result<Note, Error> {
    keyset::issue_note(
        rng,
        &keypair.secret_key,
        &Asset {
            policy_id,
            asset_name,
        },
        amount,
    )
}

pub fn refresh(
    transaction: &Refresh,
    keypair: Keypair,
    database: &Database,
) -> Result<Response, Error> {
    transaction.verify()?;

    let output_count = (0..transaction.atoms.len())
        .filter(|&i| transaction.is_output(i))
        .count();

    // The node signs only blinded points. If it signed the commitment of
    // an output, it could link the output to the note when it is spent.
    if transaction.blinded_points.len() != output_count {
        return Err(Error::InvalidOperation {
            reason: format!(
                "blinded_points length {} does not match output count {}",
                transaction.blinded_points.len(),
                output_count,
            ),
        });
    }

    for (i, atom) in transaction.atoms.iter().enumerate() {
        if atom.delegate != keypair.public_key {
            return Err(Error::InvalidAtom {
                reason: format!("Atom {} is for a different delegate", i),
            });
        }

        if transaction.is_output(i) && atom.nonce != Hash::zero() {
            return Err(Error::InvalidAtom {
                reason: format!("Output atom {} must not show its nonce", i),
            });
        }
    }

    let mut rng = rand::rng();
    let mut outputs = Vec::with_capacity(output_count);
    let mut output_idx = 0usize;
    let w = database.write()?;

    {
        let mut table = w.open_table(NOTES)?;

        for (i, atom) in transaction.atoms.iter().enumerate() {
            let asset = &transaction.asset_ids[atom.asset_id as usize];

            if transaction.is_output(i) {
                let point =
                    transaction.blinded_points[output_idx].to_point()?;
                let sig = keyset::sign_blinded(
                    &mut rng,
                    &keypair.secret_key,
                    asset,
                    atom.amount,
                    &point,
                )?;

                outputs.push(sig);
                output_idx += 1;
                continue;
            }

            // Refresh::verify() checks that each input has a signature
            // index in range.
            let signature = match atom.signature {
                Some(s) => transaction.signatures[s as usize],
                None => {
                    return Err(Error::InvalidAtom {
                        reason: format!("Atom {} is input but unsigned", i),
                    });
                }
            };

            if signature == Signature::zero() {
                return Err(Error::InvalidSignature {
                    reason: "Zero signature".to_string(),
                    signature,
                });
            }

            // Check if already spent
            if table.get(signature)?.is_some() {
                return Err(Error::AlreadySpent { signature });
            }

            // Verify before marking as spent
            let commitment = atom.commitment(&transaction.asset_ids);
            if !keyset::verify(
                &keypair.secret_key,
                asset,
                atom.amount,
                commitment.as_ref(),
                signature,
            )? {
                return Err(Error::InvalidSignature {
                    reason: format!("Atom {} has an invalid signature", i),
                    signature,
                });
            }

            // Mark as spent
            table.insert(signature, true)?;
        }
    }

    w.commit()?;
    Ok(Response::Transaction { outputs })
}

#[cfg(test)]
mod tests {
    use mugraph_core::builder::RefreshBuilder;
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    fn temp_db() -> Database {
        let path = std::env::temp_dir().join(format!(
            "mugraph-refresh-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = Database::setup(path).unwrap();
        db.migrate().unwrap();
        db
    }

    fn signed_note(keypair: &Keypair, amount: u64) -> Note {
        let mut rng = StdRng::seed_from_u64(7);
        keyset::issue_note(
            &mut rng,
            &keypair.secret_key,
            &Asset::default(),
            amount,
        )
        .expect("valid note")
    }

    #[test]
    fn refresh_with_blinded_points_produces_unblindable_signatures() {
        let mut rng = StdRng::seed_from_u64(42);
        let keypair = Keypair::random(&mut rng);
        let note = signed_note(&keypair, 128);
        let db = temp_db();

        // Build refresh: 128 -> 64 + 32 + 32
        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 96)
            .output(note.policy_id, note.asset_name, 32)
            .build()
            .unwrap();

        // Client: blind each output atom's commitment
        let secrets = refresh_tx.blind_outputs(&mut rng);

        // Server: process refresh
        let response =
            refresh(&refresh_tx, keypair, &db).expect("refresh must succeed");

        // Client: unblind and verify each output signature
        let keys = keyset::keyset(&keypair.secret_key, &Asset::default());
        match response {
            Response::Transaction { outputs } => {
                assert_eq!(outputs.len(), 3);

                for (secret, signed) in secrets.iter().zip(&outputs) {
                    let amount = refresh_tx.atoms[secret.atom_index].amount;
                    let public_key =
                        keyset::keyset_public_key(&keys, amount).unwrap();
                    let new_note = refresh_tx
                        .unblind_output(secret, signed, &public_key)
                        .expect("unblinded signature must verify");

                    assert!(
                        keyset::verify(
                            &keypair.secret_key,
                            &Asset::default(),
                            new_note.amount,
                            new_note.commitment().as_ref(),
                            new_note.signature,
                        )
                        .unwrap()
                    );
                }
            }
            other => panic!("expected Transaction response, got {:?}", other),
        }
    }

    #[test]
    fn refresh_rejects_unbalanced_transaction() {
        let mut rng = StdRng::seed_from_u64(1);
        let keypair = Keypair::random(&mut rng);
        let note = signed_note(&keypair, 8);
        let db = temp_db();

        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 8)
            .build()
            .unwrap();
        refresh_tx.blind_outputs(&mut rng);

        // break conservation: output > input
        refresh_tx.atoms[1].amount = 16;

        let result = refresh(&refresh_tx, keypair, &db);
        assert!(result.is_err(), "unbalanced refresh must be rejected");
    }

    #[test]
    fn refresh_rejects_input_signed_by_another_key() {
        let mut rng = StdRng::seed_from_u64(9);
        let keypair = Keypair::random(&mut rng);
        let other = Keypair::random(&mut rng);
        let db = temp_db();

        // A valid point, but not a signature from this node's key.
        let mut note = signed_note(&other, 8);
        note.delegate = keypair.public_key;

        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 8)
            .build()
            .unwrap();
        refresh_tx.blind_outputs(&mut rng);

        let result = refresh(&refresh_tx, keypair, &db);
        assert!(
            matches!(result, Err(Error::InvalidSignature { .. })),
            "input with a foreign signature must be rejected, got {result:?}",
        );

        let r = db.read().unwrap();
        let table = r.open_table(NOTES).unwrap();
        assert!(
            table.get(note.signature).unwrap().is_none(),
            "rejected input must not be marked as spent",
        );
    }
}
