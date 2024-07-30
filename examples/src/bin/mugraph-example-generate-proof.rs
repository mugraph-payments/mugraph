use mugraph_circuits::*;
use mugraph_core::{
    contracts::fission,
    contracts::fusion,
    crypto::{generate_keypair, schnorr::sign},
    BlindedNote, Hash, Note, SerializeBytes,
};
use rand::rngs::OsRng;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rng = &mut OsRng;

    info!("Generating server keys");
    let (server_priv, server_pub) = generate_keypair(rng);

    let nullifier = sign(rng, &server_priv, [2u8; 32].as_ref());

    let split = fission::Input {
        server_key: server_pub.compress().to_bytes(),
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 1000, // Increased this
            nullifier,
        },
        amount: 50,
    };
    info!(
        input_amount = split.input.amount,
        split_amount = split.amount,
        "Creating fission request"
    );

    let mut prover = Prover::new();

    info!("creating fission proof");
    let mut buf = [0u8; fission::Input::SIZE];
    split.to_slice(&mut buf);
    let fission_receipt = prover.prove(&buf)?;

    info!("parsing fission journal");
    let fission: fission::Output = fission::Output::from_slice(&fission_receipt.journal.bytes)?;

    info!("reading fission stdout");
    let output = BlindedNote::from_slice(&prover.stdout[..BlindedNote::SIZE])?;
    let change = BlindedNote::from_slice(&prover.stdout[BlindedNote::SIZE..])?;

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
    let input = fusion::Input {
        inputs: [output.clone(), change.clone()],
    };

    info!("Creating fusion::Output Proof");
    let mut buf = [0u8; fusion::Input::SIZE];
    input.to_slice(&mut buf);
    let fusion_receipt = prover.prove(&buf)?;

    info!("Parsing fusion journal");
    let fusion: fusion::Output = fusion::Output::from_slice(&fusion_receipt.journal.bytes)?;

    info!("Reading fusion stdout");
    let fused_output = BlindedNote::from_slice(&prover.stdout[..BlindedNote::SIZE])?;

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
