use iai::black_box;
use mugraph_core::{
    builder::{GreedyCoinSelection, TransactionBuilder},
    crypto,
    types::*,
};
use mugraph_node::{context::Context, v0::transaction_v0};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

fn setup() -> (Transaction, Context) {
    let mut rng = ChaCha20Rng::seed_from_u64(42); // Use a fixed seed for reproducibility
    let context = Context::new(&mut rng).unwrap();
    let delegate_keypair = context.keypair;

    // Create a note
    let asset_id = Hash::random(&mut rng);
    let amount = 1000;
    let note = Note {
        amount,
        delegate: delegate_keypair.public_key,
        asset_id,
        nonce: Hash::random(&mut rng),
        signature: Signature::default(),
    };

    // Blind and sign the note
    let blind = crypto::blind_note(&mut rng, &note);
    let signed = crypto::sign_blinded(&delegate_keypair.secret_key, &blind.point);
    let signature =
        crypto::unblind_signature(&signed, &blind.factor, &delegate_keypair.public_key).unwrap();
    let note = Note { signature, ..note };

    // Build a transaction
    let transaction = TransactionBuilder::new(GreedyCoinSelection)
        .input(note)
        .output(asset_id, amount / 2)
        .build()?;

    (transaction, context)
}

fn bench_transaction() {
    let (transaction, mut context) = setup();
    black_box(transaction_v0(black_box(transaction), black_box(&mut context)).unwrap());
}

iai::main!(bench_transaction);
