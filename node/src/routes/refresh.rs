use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{
        AssetId, DleqProofWithBlinding, Hash, Keypair, Note, Refresh, Response, Signature,
    },
};
use rand::{CryptoRng, RngCore};
use redb::ReadableTable;

use crate::database::{Database, NOTES};

#[inline]
pub fn emit_note<R: RngCore + CryptoRng>(
    keypair: &Keypair,
    asset_id: AssetId,
    amount: u64,
    rng: &mut R,
) -> Result<Note, Error> {
    let mut note = Note {
        delegate: keypair.public_key,
        asset_id,
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
