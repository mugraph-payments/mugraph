//! Tests for the node command line.

use std::process::Command;

use mugraph_core::types::{PublicKey, SecretKey};

/// `generate-key` must print a secret key that an operator can pass to
/// `server --secret-key`, and the public key that goes with it.
#[test]
fn generate_key_prints_a_usable_secret_key() {
    let output = Command::new(env!("CARGO_BIN_EXE_mugraph-node"))
        .arg("generate-key")
        .output()
        .expect("run mugraph-node");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let field = |name: &str| -> String {
        stdout
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
            .unwrap_or_else(|| panic!("no {name} line in {stdout:?}"))
            .to_string()
    };

    let secret: [u8; 32] = hex::decode(field("secret_key"))
        .unwrap()
        .try_into()
        .unwrap();
    let public: [u8; 32] = hex::decode(field("public_key"))
        .unwrap()
        .try_into()
        .unwrap();

    assert_eq!(SecretKey::from(secret).public(), PublicKey(public));
}
