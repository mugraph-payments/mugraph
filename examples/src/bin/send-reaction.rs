use mugraph_client::prelude::{crypto::*, *};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use tracing::*;

macro_rules! timed {
    ($name:literal, $($arg:tt)*) => {{
        let s = tracing::span!(Level::INFO, concat!("mugraph::task[", $name, "]"));
        let _e = s.enter();

        tracing::info!("Starting task");

        let now = std::time::Instant::now();
        let result = { $($arg)* };

        tracing::info!(elapsed = ?now.elapsed(), "Finished task");

        result
    }}
}

fn build_transaction() -> Result<Transaction> {
    let mut rng = rand::thread_rng();

    let (server_priv, server_pub) = generate_keypair(&mut rng);

    let mut note = Note {
        delegate: server_pub,
        asset_id: Hash::digest(b"Sample Asset"),
        nonce: Hash::digest(b"Sample Nonce"),
        amount: 100,
        signature: Signature::default(),
    };

    let (_y, _r, b_prime) = dh::blind(&mut rand::thread_rng(), note.commitment().as_ref());
    let _signed_point = dh::sign_blinded(&server_priv, &b_prime)?;

    // TODO: add unblinded signature for note
    note.signature = Signature::default();

    Ok(TransactionBuilder::new()
        .input(&note)
        .output(note.asset_id, 60)
        .output(note.asset_id, 40)
        .build())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut stdout = Vec::new();

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .write(&build_transaction()?)
            .map_err(|_| Error::ZKVM)?
            .stdout(&mut stdout)
            .build()
            .map_err(|_| Error::ZKVM)?
    });

    // Obtain the default prover.
    let prover = timed!("create prover", default_prover());

    // Produce a receipt by proving the specified ELF binary.
    let _ = timed!(
        "prove transaction",
        prover
            .prove_with_opts(env, VALIDATE_ELF, &ProverOpts::fast())
            .map_err(|_| Error::ZKVM)?
            .receipt
    );
    let cycles: u64 = risc0_zkvm::serde::from_slice(&stdout)?;
    info!("Done, proof took {} cycles.", cycles);

    Ok(())
}
