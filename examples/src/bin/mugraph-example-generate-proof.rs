use mugraph_circuits::*;
use mugraph_core::{BlindedNote, Fission, Hash, Note, Split};
use mugraph_crypto::{generate_keypair, schnorr::sign};
use rand::rngs::OsRng;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rng = &mut OsRng;

    let (server_priv, server_pub) = generate_keypair(rng);
    let nullifier = sign(rng, &server_priv, [2u8; 32].as_ref());

    let request = Split {
        server_key: server_pub.compress().to_bytes(),
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 100,
            nullifier,
        },
        amount: 50,
    };

    let mut prover = Prover::new();
    let receipt = prover.prove(&request)?;

    let fission: Fission = receipt.journal.decode()?;

    let (output, change): (BlindedNote, BlindedNote) = prover.read()?;
    let (so, sc) = (
        sign(rng, &server_priv, output.blinded_secret.as_ref()),
        sign(rng, &server_priv, change.blinded_secret.as_ref()),
    );

    let (output, change) = (output.unblind(so), change.unblind(sc));

    println!(
        "Spend:\n\n{}",
        serde_json::to_string_pretty(&fission).unwrap()
    );
    println!(
        "Output:\n\n{}",
        serde_json::to_string_pretty(&output).unwrap()
    );
    println!(
        "Change:\n\n{}",
        serde_json::to_string_pretty(&change).unwrap()
    );

    Ok(())
}
