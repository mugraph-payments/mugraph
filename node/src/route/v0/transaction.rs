use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{Signature, Transaction, V0Response},
};

use crate::context::Context;

#[inline]
pub fn transaction_v0(
    transaction: Transaction,
    ctx: &mut Context,
) -> Result<V0Response, Vec<Error>> {
    let mut outputs = vec![];
    let mut errors = vec![];

    for atom in transaction.atoms.iter() {
        match atom.is_input() {
            true => {
                let signature = match atom.signature {
                    Some(s) if transaction.signatures[s as usize] == Signature::zero() => {
                        errors.push(Error::InvalidSignature {
                            reason: "Signature can not be empty".to_string(),
                            signature: Signature::zero(),
                        });

                        Signature::zero()
                    }
                    Some(s) => transaction.signatures[s as usize],
                    None => {
                        errors.push(Error::InvalidAtom {
                            reason: "Atom {} is an input but it is not signed.".into(),
                        });

                        Signature::zero()
                    }
                };

                let table = ctx.db_read().expect("Failed to read database table");

                match crypto::verify(&ctx.keypair.public_key, atom.nonce.as_ref(), signature) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(Error::InvalidSignature {
                            reason: e.to_string(),
                            signature,
                        });
                    }
                }

                match table.get(signature.0) {
                    Ok(Some(_)) => {
                        errors.push(Error::AlreadySpent { signature });
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(Error::ServerError {
                            reason: e.to_string(),
                        });
                    }
                }
            }
            false => {
                let sig = crypto::sign_blinded(
                    &ctx.keypair.secret_key,
                    &crypto::hash_to_curve(atom.commitment(&transaction.asset_ids).as_ref()),
                );

                outputs.push(sig);
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(V0Response::Transaction { outputs })
}
