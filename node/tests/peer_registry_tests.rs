use std::io::Write;

use mugraph_node::peer_registry::PeerRegistry;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn write_temp(contents: &str) -> Result<tempfile::NamedTempFile, std::io::Error> {
    let mut f = tempfile::NamedTempFile::new()?;
    f.write_all(contents.as_bytes())?;
    Ok(f)
}

#[test]
fn loads_valid_json_registry() -> TestResult {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "revoked": false
    },
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k2",
      "public_key_hex": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "revoked": false
    }
  ]
}"#,
    )?;

    let registry = PeerRegistry::load(file.path())?;
    assert_eq!(registry.peers.len(), 2);

    registry.validate()?;
    Ok(())
}

#[test]
fn rejects_duplicate_node_and_kid() -> TestResult {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "revoked": false
    },
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "revoked": false
    }
  ]
}"#,
    )?;

    let registry = PeerRegistry::load(file.path())?;
    let err = registry.validate().expect_err("must reject duplicate node+kid");
    assert!(err.to_string().contains("duplicate"));
    Ok(())
}

#[test]
fn rejects_non_ed25519_auth_alg() -> TestResult {
    let file = write_temp(
        r#"{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ristretto",
      "kid": "alpha-k1",
      "public_key_hex": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "revoked": false
    }
  ]
}"#,
    )?;

    let registry = PeerRegistry::load(file.path())?;
    let err = registry.validate().expect_err("must reject non-ed25519 auth alg");
    assert!(err.to_string().contains("Ed25519"));
    Ok(())
}
