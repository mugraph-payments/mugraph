use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{Hash, Keypair, Note, Refresh, Response, Signature},
};
use rand::{CryptoRng, RngCore};
use redb::ReadableTable;

use crate::database::{Database, NOTES};

#[inline]
pub fn emit_note<R: RngCore + CryptoRng>(
    keypair: &Keypair,
    asset_id: Hash,
    amount: u64,
    rng: &mut R,
) -> Result<Note, Error> {
    let mut note = Note {
        delegate: keypair.public_key,
        asset_id,
        nonce: Hash::random(rng),
        amount,
        signature: Signature::default(),
    };

    let blind = crypto::blind_note(rng, &note);
    let signed = crypto::sign_blinded(&keypair.secret_key, &blind.point);
    note.signature =
        crypto::unblind_signature(&signed, &blind.factor, &keypair.public_key)?;

    Ok(note)
}

pub fn refresh(
    transaction: &Refresh,
    keypair: Keypair,
    database: &Database,
) -> Result<Response, Error> {
    let mut outputs =
        Vec::with_capacity(transaction.input_mask.count_zeros() as usize);
    let w = database.write()?;

    {
        let mut table = w.open_table(NOTES)?;

        for (i, atom) in transaction.atoms.iter().enumerate() {
            if transaction.is_output(i) {
                let sig = crypto::sign_blinded(
                    &keypair.secret_key,
                    &crypto::hash_to_curve(
                        atom.commitment(&transaction.asset_ids).as_ref(),
                    ),
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
