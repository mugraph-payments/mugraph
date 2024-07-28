use mugraph_circuits::*;
use mugraph_core::{Error, Fission, Hash, Note, Result, Split};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let request = Split {
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 100,
            nullifier: Hash([2u8; 32]),
        },
        amount: 50,
    };

    let mut prover = Prover::new();
    let receipt = prover.prove(&request)?;

    let fission: Fission = receipt.journal.decode().map_err(|_| Error::JournalDecode)?;
    let (output, change): (Note, Note) = prover.read()?;

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
