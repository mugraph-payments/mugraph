use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    error::Error,
    types::{Hash, Keypair, Note, Response, Signature},
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
    let mut note = Note {
        delegate: keypair.public_key,
        policy_id: Default::default(),
        asset_name: Default::default(),
        nonce: Hash::random(&mut rng),
        amount,
        signature: Signature::default(),
        dleq: None,
    };

    let blind = crypto::blind_note(&mut rng, &note);
    let signed =
        crypto::sign_blinded(&mut rng, &keypair.secret_key, &blind.point);
    note.signature = crypto::unblind_signature(
        &signed.signature,
        &blind.factor,
        &keypair.public_key,
    )
    .expect("valid unblind");
    note
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
    let note = signed_note(&keypair, 10);
    let (_dir, db) = temp_db();

    let refresh_tx = RefreshBuilder::new()
        .input(note.clone())
        .output(note.policy_id, note.asset_name, 10)
        .build()
        .unwrap();

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
fn refresh_rejects_already_spent_note_without_extra_writes() {
    let mut rng = StdRng::seed_from_u64(43);
    let keypair = Keypair::random(&mut rng);
    let note = signed_note(&keypair, 10);
    let (_dir, db) = temp_db();

    {
        let write_tx = db.write().unwrap();
        {
            let mut table = write_tx.open_table(NOTES).unwrap();
            table.insert(note.signature, true).unwrap();
        }
        write_tx.commit().unwrap();
    }

    let refresh_tx = RefreshBuilder::new()
        .input(note.clone())
        .output(note.policy_id, note.asset_name, 10)
        .build()
        .unwrap();

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
    let note = signed_note(&keypair, 10);
    let (_dir, db) = temp_db();

    let mut refresh_tx = RefreshBuilder::new()
        .input(note.clone())
        .output(note.policy_id, note.asset_name, 10)
        .build()
        .unwrap();
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
    let note1 = signed_note(&keypair, 7);
    let note2 = signed_note(&keypair, 5);
    let (_dir, db) = temp_db();

    {
        let write_tx = db.write().unwrap();
        {
            let mut table = write_tx.open_table(NOTES).unwrap();
            table.insert(note2.signature, true).unwrap();
        }
        write_tx.commit().unwrap();
    }

    let refresh_tx = RefreshBuilder::new()
        .input(note1.clone())
        .input(note2.clone())
        .output(note1.policy_id, note1.asset_name, 12)
        .build()
        .unwrap();

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
