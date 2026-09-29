use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    error::Error,
    keyset,
    types::{Asset, Hash, Keypair, Note, Refresh, Response, Signature},
};
use mugraph_node::{
    database::{Database, NOTES},
    routes::refresh,
};
use rand::{SeedableRng, rngs::StdRng};
use redb::ReadableTable;
use tempfile::TempDir;

fn temp_db() -> (TempDir, Database) {
    let dir = TempDir::new().unwrap();
    let db = Database::setup(dir.path().join("db.redb")).unwrap();
    db.migrate().unwrap();
    (dir, db)
}

fn signed_note(keypair: &Keypair, amount: u64) -> Note {
    let mut rng = StdRng::seed_from_u64(7 + amount);
    keyset::issue_note(&mut rng, &keypair.secret_key, &Asset::default(), amount)
        .expect("valid note")
}

/// Builds a refresh with blinded outputs, as a wallet does.
fn blinded_refresh(
    inputs: &[&Note],
    output_amount: u64,
    rng: &mut StdRng,
) -> (Refresh, Vec<mugraph_core::types::OutputSecret>) {
    let mut builder = RefreshBuilder::new();
    for note in inputs {
        builder = builder.input((*note).clone());
    }
    let mut refresh_tx = builder
        .output(inputs[0].policy_id, inputs[0].asset_name, output_amount)
        .build()
        .unwrap();
    let secrets = refresh_tx.blind_outputs(rng);
    (refresh_tx, secrets)
}

fn note_row_count(db: &Database) -> usize {
    let read_tx = db.read().unwrap();
    let table = read_tx.open_table(NOTES).unwrap();
    table.iter().unwrap().count()
}

#[test]
fn refresh_success_returns_output_and_marks_inputs_spent() {
    let mut rng = StdRng::seed_from_u64(42);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    let (refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);

    let response =
        refresh(&refresh_tx, keypair, &db).expect("refresh accepted");
    let Response::Transaction { outputs } = response else {
        panic!("expected refresh transaction response");
    };
    assert_eq!(outputs.len(), 1);

    let read_tx = db.read().unwrap();
    let table = read_tx.open_table(NOTES).unwrap();
    assert!(table.get(note.signature).unwrap().is_some());
    assert_eq!(
        table.iter().unwrap().count(),
        2,
        "zero marker + spent input"
    );
}

#[test]
fn refresh_outputs_verify_with_the_denomination_key() {
    let mut rng = StdRng::seed_from_u64(46);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    // Refresh 8 into 6, which the builder splits into outputs of 4 and 2,
    // plus a second output of 2 for the change.
    let (refresh_tx, secrets) = {
        let mut refresh_tx = RefreshBuilder::new()
            .input(note.clone())
            .output(note.policy_id, note.asset_name, 6)
            .output(note.policy_id, note.asset_name, 2)
            .build()
            .unwrap();
        let secrets = refresh_tx.blind_outputs(&mut rng);
        (refresh_tx, secrets)
    };

    let Response::Transaction { outputs } =
        refresh(&refresh_tx, keypair, &db).expect("refresh accepted")
    else {
        panic!("expected refresh transaction response");
    };

    let keys = keyset::keyset(&keypair.secret_key, &Asset::default());
    for (secret, signed) in secrets.iter().zip(&outputs) {
        let amount = refresh_tx.atoms[secret.atom_index].amount;
        let public_key = keyset::keyset_public_key(&keys, amount).unwrap();

        let new_note = refresh_tx
            .unblind_output(secret, signed, &public_key)
            .expect("output verifies with its denomination key");
        assert!(
            refresh_tx
                .unblind_output(secret, signed, &keypair.public_key)
                .is_err(),
            "output must not verify with the master key"
        );
        assert_eq!(new_note.amount, amount);
    }
}

#[test]
fn refresh_rejects_output_that_shows_its_nonce() {
    let mut rng = StdRng::seed_from_u64(47);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    let (mut refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);
    let output = refresh_tx.atoms.len() - 1;
    refresh_tx.atoms[output].nonce = Hash::random(&mut rng);

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(matches!(err, Error::InvalidAtom { .. }), "got {err:?}");
    assert_eq!(note_row_count(&db), 1, "only zero marker should remain");
}

#[test]
fn refresh_rejects_outputs_without_blinded_points() {
    let mut rng = StdRng::seed_from_u64(48);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    let (mut refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);
    refresh_tx.blinded_points.clear();

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(matches!(err, Error::InvalidOperation { .. }), "got {err:?}");
    assert_eq!(note_row_count(&db), 1, "only zero marker should remain");
}

#[test]
fn refresh_rejects_input_signed_with_the_master_key() {
    let mut rng = StdRng::seed_from_u64(49);
    let keypair = Keypair::random(&mut rng);
    let (_dir, db) = temp_db();

    // A note signed with the master key has no fixed value, so the node
    // must not accept it.
    let mut note = Note {
        amount: 8,
        delegate: keypair.public_key,
        nonce: Hash::random(&mut rng),
        ..Default::default()
    };
    let blinded = crypto::blind_note(&mut rng, &note);
    let signed =
        crypto::sign_blinded(&mut rng, &keypair.secret_key, &blinded.point);
    note.signature = crypto::unblind_signature(
        &signed.signature,
        &blinded.factor,
        &keypair.public_key,
    )
    .unwrap();

    let (refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(matches!(err, Error::InvalidSignature { .. }), "got {err:?}");
    assert_eq!(note_row_count(&db), 1, "only zero marker should remain");
}

#[test]
fn refresh_rejects_already_spent_note_without_extra_writes() {
    let mut rng = StdRng::seed_from_u64(43);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    {
        let write_tx = db.write().unwrap();
        {
            let mut table = write_tx.open_table(NOTES).unwrap();
            table.insert(note.signature, true).unwrap();
        }
        write_tx.commit().unwrap();
    }

    let (refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(
        matches!(err, Error::AlreadySpent { signature } if signature == note.signature)
    );
    assert_eq!(
        note_row_count(&db),
        2,
        "zero marker + pre-seeded spent note"
    );
}

#[test]
fn refresh_rejects_invalid_signature_without_burning_note() {
    let mut rng = StdRng::seed_from_u64(44);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 8);
    let (_dir, db) = temp_db();

    let (mut refresh_tx, _) = blinded_refresh(&[&note], 8, &mut rng);
    refresh_tx.signatures[0] = Signature::from([0x55u8; 32]);

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(matches!(err, Error::InvalidSignature { .. }));

    let read_tx = db.read().unwrap();
    let table = read_tx.open_table(NOTES).unwrap();
    assert!(table.get(note.signature).unwrap().is_none());
    assert_eq!(
        table.iter().unwrap().count(),
        1,
        "only zero marker should remain"
    );
}

#[test]
fn refresh_is_atomic_when_a_later_input_is_already_spent() {
    let mut rng = StdRng::seed_from_u64(45);
    let keypair = Keypair::random(&mut rng);
    let note1 = signed_note(&keypair, 8);
    let note2 = signed_note(&keypair, 4);
    let (_dir, db) = temp_db();

    {
        let write_tx = db.write().unwrap();
        {
            let mut table = write_tx.open_table(NOTES).unwrap();
            table.insert(note2.signature, true).unwrap();
        }
        write_tx.commit().unwrap();
    }

    let (refresh_tx, _) = blinded_refresh(&[&note1, &note2], 12, &mut rng);

    let err = refresh(&refresh_tx, keypair, &db).unwrap_err();
    assert!(
        matches!(err, Error::AlreadySpent { signature } if signature == note2.signature)
    );

    let read_tx = db.read().unwrap();
    let table = read_tx.open_table(NOTES).unwrap();
    assert!(table.get(note1.signature).unwrap().is_none());
    assert!(table.get(note2.signature).unwrap().is_some());
    assert_eq!(
        table.iter().unwrap().count(),
        2,
        "zero marker + pre-seeded spent note"
    );
}
