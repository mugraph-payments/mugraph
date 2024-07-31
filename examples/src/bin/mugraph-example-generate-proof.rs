use mugraph_core::{
    crypto::{generate_keypair, schnorr::sign},
    programs::fission,
    programs::fusion,
    run_program, Hash, Note,
};
use mugraph_programs::*;
use rand::rngs::OsRng;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rng = &mut OsRng;

    info!("[server] Generating server keys");
    let (server_priv, server_pub) = generate_keypair(rng);

    let nullifier = sign(rng, &server_priv, [2u8; 32].as_ref());

    let stdin = fission::Input {
        input: Note {
            server_key: server_pub.compress().to_bytes(),
            asset_id: Hash([1u8; 32]),
            amount: 1000,
            nullifier,
        },
        amount: 50,
    };

    let (stdout, _) = run_program!(fission, stdin)?;

    info!("[server] signing outputs");
    let (so, sc) = (
        sign(rng, &server_priv, stdout.output.secret.as_ref()),
        sign(rng, &server_priv, stdout.change.secret.as_ref()),
    );

    info!("Unblinding fission tokens");
    let (output, change) = (stdout.output.unblind(so), stdout.change.unblind(sc));

    println!(
        "Unblinded fission outputs:\n\n- Output:\n{}\n- Change:\n{}",
        serde_json::to_string_pretty(&output).unwrap(),
        serde_json::to_string_pretty(&change).unwrap()
    );

    info!("Creating fusion request");
    let stdin = fusion::Input {
        a: output,
        b: change,
    };

    let (stdout, _) = run_program!(fusion, stdin)?;

    info!("[server] signing output");
    let sf = sign(rng, &server_priv, stdout.note.secret.as_ref());

    info!("Unblinding fusion token");
    let output = stdout.note.unblind(sf);

    println!(
        "Unblinded fusion output:\n\n{}",
        serde_json::to_string_pretty(&output).unwrap()
    );

    Ok(())
}
