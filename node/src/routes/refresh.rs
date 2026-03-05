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
    note.signature =
        crypto::unblind_signature(&signed.signature, &blind.factor, &keypair.public_key)?;
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

    let mut rng = rand::rng();
    let mut outputs = Vec::with_capacity(transaction.input_mask.count_zeros() as usize);
    let w = database.write()?;

    {
        let mut table = w.open_table(NOTES)?;

        for (i, atom) in transaction.atoms.iter().enumerate() {
            if transaction.is_output(i) {
                let sig = crypto::sign_blinded(
                    &mut rng,
                    &keypair.secret_key,
                    &crypto::hash_to_curve(atom.commitment(&transaction.asset_ids).as_ref()),
                );

                outputs.push(sig);
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
            crypto::verify(&keypair.public_key, commitment.as_ref(), signature)?;

            // Mark as spent
            table.insert(signature, true)?;
        }
    }

    w.commit()?;
    Ok(Response::Transaction { outputs })
}

#[cfg(test)]
mod tests {
    use mugraph_core::{builder::RefreshBuilder, crypto, types::{Hash, Note}};
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
        let signed = crypto::sign_blinded(&mut rng, &keypair.secret_key, &blind.point);
        note.signature = crypto::unblind_signature(&signed.signature, &blind.factor, &keypair.public_key)
            .expect("valid unblind");
        note
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
