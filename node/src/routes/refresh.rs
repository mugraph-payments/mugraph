use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{
        AssetName,
        DleqProofWithBlinding,
        Hash,
        Keypair,
        Note,
        PolicyId,
        Refresh,
        Response,
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
    let mut note = Note {
        delegate: keypair.public_key,
        policy_id,
        asset_name,
        nonce: Hash::random(rng),
        amount,
        signature: Signature::default(),
        dleq: None,
    };

    let blind = crypto::blind_note(rng, &note);
    let signed = crypto::sign_blinded(rng, &keypair.secret_key, &blind.point);
    note.signature = crypto::unblind_signature(
        &signed.signature,
        &blind.factor,
        &keypair.public_key,
    )?;
    note.dleq = Some(DleqProofWithBlinding {
        proof: signed.proof,
        blinding_factor: blind.factor.into(),
    });

    Ok(note)
}

pub fn refresh(
    transaction: &Refresh,
    keypair: Keypair,
    database: &Database,
) -> Result<Response, Error> {
    transaction.verify()?;

    let output_count = transaction
        .atoms
        .iter()
        .enumerate()
        .filter(|(i, _)| transaction.is_output(*i))
        .count();

    // Validate blinded_points length when provided
    if !transaction.blinded_points.is_empty()
        && transaction.blinded_points.len() != output_count
    {
        return Err(Error::InvalidOperation {
            reason: format!(
                "blinded_points length {} does not match output count {}",
                transaction.blinded_points.len(),
                output_count,
            ),
        });
    }

    let mut rng = rand::rng();
    let mut outputs = Vec::with_capacity(output_count);
    let mut output_idx = 0usize;
    let w = database.write()?;

    {
        let mut table = w.open_table(NOTES)?;

        for (i, atom) in transaction.atoms.iter().enumerate() {
            if transaction.is_output(i) {
                let point = if let Some(bp) =
                    transaction.blinded_points.get(output_idx)
                {
                    bp.to_point()?
                } else {
                    crypto::hash_to_curve(
                        atom.commitment(&transaction.asset_ids).as_ref(),
                    )
                };

                let sig =
                    crypto::sign_blinded(&mut rng, &keypair.secret_key, &point);

                outputs.push(sig);
                output_idx += 1;
                continue;
            }

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
            crypto::verify(
                &keypair.public_key,
                commitment.as_ref(),
                signature,
            )?;

            // Mark as spent
            table.insert(signature, true)?;
        }
    }

    w.commit()?;
    Ok(Response::Transaction { outputs })
}

#[cfg(test)]
mod tests {
    use mugraph_core::{
        builder::RefreshBuilder,
        crypto,
        types::{Hash, Note},
    };
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
        let mut note = Note {
            delegate: keypair.public_key,
            policy_id: Default::default(),
            asset_name: Default::default(),
            nonce: Hash::random(&mut rng),
            amount,
            signature: Signature::default(),
            dleq: None,
        };

        let blind = crypto::blind_note(&mut rng, &note);
        let signed =
            crypto::sign_blinded(&mut rng, &keypair.secret_key, &blind.point);
        note.signature = crypto::unblind_signature(
            &signed.signature,
            &blind.factor,
            &keypair.public_key,
        )
        .expect("valid unblind");
        note
    }

    #[test]
    fn refresh_with_blinded_points_produces_unblindable_signatures() {
        let mut rng = StdRng::seed_from_u64(42);
        let keypair = Keypair::random(&mut rng);
        let note = signed_note(&keypair, 100);
        let db = temp_db();

        // Build refresh: 100 -> 60 + 40
        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 60)
            .output(note.policy_id, note.asset_name, 40)
            .build()
            .unwrap();

        // Client: blind each output atom's commitment
        let mut blinding_factors = Vec::new();
        let mut blinded_points = Vec::new();
        for (i, atom) in refresh_tx.atoms.iter().enumerate() {
            if refresh_tx.is_output(i) {
                let commitment = atom.commitment(&refresh_tx.asset_ids);
                let blinded = crypto::blind(&mut rng, commitment.as_ref());
                blinding_factors.push(blinded.factor);
                blinded_points.push(Signature::from(blinded.point));
            }
        }
        refresh_tx.blinded_points = blinded_points;

        // Server: process refresh
        let response =
            refresh(&refresh_tx, keypair, &db).expect("refresh must succeed");

        // Client: unblind and verify each output signature
        match response {
            Response::Transaction { outputs } => {
                assert_eq!(outputs.len(), 2);

                let mut output_idx = 0;
                for (i, atom) in refresh_tx.atoms.iter().enumerate() {
                    if refresh_tx.is_output(i) {
                        let commitment = atom.commitment(&refresh_tx.asset_ids);
                        let sig = &outputs[output_idx];
                        let r = &blinding_factors[output_idx];

                        // Unblind the signature
                        let unblinded = crypto::unblind_signature(
                            &sig.signature,
                            r,
                            &keypair.public_key,
                        )
                        .expect("unblind must succeed");

                        // Verify the unblinded signature against the
                        // commitment
                        assert!(
                            crypto::verify(
                                &keypair.public_key,
                                commitment.as_ref(),
                                unblinded,
                            )
                            .expect("verify must not error"),
                            "unblinded signature must verify for output {}",
                            output_idx,
                        );

                        output_idx += 1;
                    }
                }
            }
            other => panic!("expected Transaction response, got {:?}", other),
        }
    }

    #[test]
    fn refresh_rejects_unbalanced_transaction() {
        let mut rng = StdRng::seed_from_u64(42);
        let keypair = Keypair::random(&mut rng);
        let note = signed_note(&keypair, 10);
        let db = temp_db();

        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 10)
            .build()
            .unwrap();

        // break conservation: output > input
        refresh_tx.atoms[1].amount = 11;

        let result = refresh(&refresh_tx, keypair, &db);
        assert!(result.is_err(), "unbalanced refresh must be rejected");
    }
}
