use std::io::Write;

use mugraph_node::peer_registry::PeerRegistry;

fn write_temp(contents: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    file
}

#[test]
fn load_rejects_missing_registry_file() {
    let missing = std::env::temp_dir().join(format!(
        "missing-peer-registry-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let err = PeerRegistry::load(&missing).unwrap_err();
    assert!(err.to_string().contains("failed to read peer registry"));
}

#[test]
fn load_rejects_malformed_registry_json() {
    let file = write_temp("{not-json");

    let err = PeerRegistry::load(file.path()).unwrap_err();
    assert!(err.to_string().contains("invalid peer registry JSON"));
}

#[test]
fn validate_rejects_invalid_endpoint_url() {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "not a url",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "revoked": false
    }
  ]
}"#,
    );

    let registry = PeerRegistry::load(file.path()).unwrap();
    let err = registry.validate().unwrap_err();
    assert!(err.to_string().contains("endpoint is not a valid URL"));
}

#[test]
fn validate_rejects_invalid_public_key_hex() {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "not-hex",
      "revoked": false
    }
  ]
}"#,
    );

    let registry = PeerRegistry::load(file.path()).unwrap();
    let err = registry.validate().unwrap_err();
    assert!(err.to_string().contains("public_key_hex is not valid hex"));
}

#[test]
fn validate_rejects_wrong_length_public_key() {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "revoked": false
    }
  ]
}"#,
    );

    let registry = PeerRegistry::load(file.path()).unwrap();
    let err = registry.validate().unwrap_err();
    assert!(err.to_string().contains("public_key_hex must be 32 bytes"));
}
