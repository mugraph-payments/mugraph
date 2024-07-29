use mugraph_circuits::*;
use mugraph_core::{BlindedNote, Fission, Fusion, Hash, Join, Note, Split};
use mugraph_crypto::{generate_keypair, schnorr::sign};
use rand::rngs::OsRng;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rng = &mut OsRng;

    info!("Generating server keys");
    let (server_priv, server_pub) = generate_keypair(rng);

    info!("Creating fission request");
    let nullifier = sign(rng, &server_priv, [2u8; 32].as_ref());

    let fission_request = Split {
        server_key: server_pub.compress().to_bytes(),
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 100,
            nullifier,
        },
        amount: 50,
    };

    let mut prover = Prover::new();

    info!("Creating Fission Proof");
    let fission_receipt = prover.prove(&fission_request)?;

    info!("Parsing fission journal");
    let fission: Fission = fission_receipt.journal.decode()?;

    info!("Reading fission stdout");
    let (output, change): (BlindedNote, BlindedNote) = prover.read()?;

    info!("[server] signing outputs");
    let (so, sc) = (
        sign(rng, &server_priv, output.secret.as_ref()),
        sign(rng, &server_priv, change.secret.as_ref()),
    );

    info!("Unblinding fission tokens");
    let (output, change) = (output.unblind(so), change.unblind(sc));

    println!(
        "Fission:\n\n{}",
        serde_json::to_string_pretty(&fission).unwrap()
    );
    println!(
        "Fission Output:\n\n{}",
        serde_json::to_string_pretty(&output).unwrap()
    );
    println!(
        "Fission Change:\n\n{}",
        serde_json::to_string_pretty(&change).unwrap()
    );

    info!("Creating fusion request");
    let fusion_request = Join {
        inputs: [output.clone(), change.clone()],
    };

    info!("Creating Fusion Proof");
    let fusion_receipt = prover.prove(&fusion_request)?;

    info!("Parsing fusion journal");
    let fusion: Fusion = fusion_receipt.journal.decode()?;

    info!("Reading fusion stdout");
    let fused_output: BlindedNote = prover.read()?;

    info!("[server] signing output");
    let sf = sign(rng, &server_priv, fused_output.secret.as_ref());

    info!("Unblinding fusion token");
    let fused_output = fused_output.unblind(sf);

    println!(
        "Fusion:\n\n{}",
        serde_json::to_string_pretty(&fusion).unwrap()
    );

    println!(
        "Fusion Output:\n\n{}",
        serde_json::to_string_pretty(&fused_output).unwrap()
    );

    Ok(())
}
