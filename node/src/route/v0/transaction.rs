use color_eyre::eyre::Result;
use mugraph_core::{
    crypto::{self, schnorr::SchnorrPair, traits::Pair},
    error::Error,
    types::{Keypair, SecretKey, Signature, Transaction, V0Response},
};

use crate::database::{Database, NOTES};

#[inline]
pub fn transaction_v0<P: Pair>(
    transaction: &Transaction,
    keypair: &P,
    database: &mut Database,
) -> Result<V0Response, Error> {
    let mut outputs = Vec::with_capacity(transaction.input_mask.count_zeros() as usize);
    let mut consumed_inputs = Vec::with_capacity(transaction.input_mask.count_ones() as usize);

    let w = database.write()?;
    let read = database.read()?.open_table(NOTES)?;

    {
        for (i, atom) in transaction.atoms.iter().enumerate() {
            if transaction.is_output(i) {
                let sig = crypto::sign_blinded::<P>(
                    &keypair.secret(),
                    &crypto::hash_to_curve(atom.commitment(&transaction.asset_ids).as_ref()),
                );

                outputs.push(sig);

                continue;
            }

            let signature = match atom.signature {
                Some(s) if transaction.signatures[s as usize] == Signature::zero() => {
                    return Err(Error::InvalidSignature {
                        reason: "Signature can not be empty".to_string(),
                        signature: Signature::zero(),
                    });
                }
                Some(s) => transaction.signatures[s as usize],
                None => {
                    return Err(Error::InvalidAtom {
                        reason: "Atom {} is an input but it is not signed.".into(),
                    });
                }
            };

            crypto::verify::<P>(&keypair.public(), atom.nonce.as_ref(), signature)?;

            match read.get(signature) {
                Ok(Some(_)) => {
                    return Err(Error::AlreadySpent { signature });
                }
                Ok(None) => {
                    consumed_inputs.push(signature);
                }
                Err(e) => {
                    return Err(Error::ServerError {
                        reason: e.to_string(),
                    });
                }
            }
        }

        let mut table = w.open_table(NOTES)?;

        for input in consumed_inputs.into_iter() {
            table.insert(input, true)?;
        }
    }

    w.commit()?;

    Ok(V0Response::Transaction { outputs })
}
