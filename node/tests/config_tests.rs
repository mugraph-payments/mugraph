//! Tests for node configuration

use clap::Parser;
use mugraph_node::config::Config;

fn parse_server(args: &[&str]) -> Config {
    Config::try_parse_from(std::iter::once("mugraph-node").chain(std::iter::once("server")).chain(args.iter().copied()))
        .expect("config should parse")
}

#[test]
fn parse_server_applies_cli_defaults() {
    let config = parse_server(&[]);

    assert_eq!(config.network(), "preprod");
    assert_eq!(config.provider_type(), "blockfrost");
    assert_eq!(config.deposit_confirm_depth(), 15);
    assert_eq!(config.deposit_expiration_blocks(), 1440);
    assert_eq!(config.min_deposit_value(), 1_000_000);
    assert_eq!(config.max_tx_size(), 16_384);
    assert_eq!(config.max_withdrawal_fee(), 2_000_000);
    assert_eq!(config.fee_tolerance_pct(), 5);
    assert!(!config.dev_mode());
}

#[test]
fn parse_server_prefers_explicit_cli_overrides() {
    let config = parse_server(&[
        "--cardano-network",
        "mainnet",
        "--cardano-provider",
        "maestro",
        "--cardano-api-key",
        "maestro_key",
        "--cardano-provider-url",
        "https://custom.api.com",
        "--cardano-payment-sk",
        "deadbeef",
        "--xnode-peer-registry-file",
        "/tmp/peers.json",
        "--deposit-confirm-depth",
        "20",
        "--deposit-expiration-blocks",
        "2000",
        "--min-deposit-value",
        "2000000",
        "--max-tx-size",
        "32768",
        "--max-withdrawal-fee",
        "3000000",
        "--fee-tolerance-pct",
        "10",
        "--dev-mode",
    ]);

    assert_eq!(config.network(), "mainnet");
    assert_eq!(config.provider_type(), "maestro");
    assert_eq!(config.provider_api_key(), "maestro_key");
    assert_eq!(config.provider_url(), Some("https://custom.api.com".to_string()));
    assert_eq!(config.payment_sk(), Some("deadbeef".to_string()));
    assert_eq!(
        config.xnode_peer_registry_file(),
        Some("/tmp/peers.json".to_string())
    );
    assert_eq!(config.deposit_confirm_depth(), 20);
    assert_eq!(config.deposit_expiration_blocks(), 2000);
    assert_eq!(config.min_deposit_value(), 2_000_000);
    assert_eq!(config.max_tx_size(), 32_768);
    assert_eq!(config.max_withdrawal_fee(), 3_000_000);
    assert_eq!(config.fee_tolerance_pct(), 10);
    assert!(config.dev_mode());
}

#[test]
fn keypair_accepts_valid_secret_key_hex() {
    let config = parse_server(&["--secret-key", &muhex::encode([7u8; 32])]);
    let keypair = config.keypair().expect("valid secret key");

    assert_eq!(*keypair.secret_key, [7u8; 32]);
    assert_eq!(keypair.public_key, keypair.secret_key.public());
}

#[test]
fn keypair_rejects_malformed_secret_key_hex() {
    let config = parse_server(&["--secret-key", "zz-not-hex"]);
    let err = config.keypair().expect_err("malformed hex must fail");

    assert!(matches!(err, mugraph_core::error::Error::InvalidKey { .. }));
}

#[test]
fn keypair_rejects_wrong_length_secret_key_hex() {
    let config = parse_server(&["--secret-key", &muhex::encode([7u8; 31])]);
    let err = config.keypair().expect_err("wrong-length hex must fail");

    assert!(format!("{err:?}").contains("Secret key must be 32 bytes"));
}

#[test]
fn keypair_is_deterministic_for_same_seed() {
    let a = parse_server(&["--seed", "42"])
        .keypair()
        .expect("seeded keypair a");
    let b = parse_server(&["--seed", "42"])
        .keypair()
        .expect("seeded keypair b");

    assert_eq!(*a.secret_key, *b.secret_key);
    assert_eq!(a.public_key, b.public_key);
}

#[test]
fn keypair_prefers_secret_key_over_seed() {
    let secret_hex = muhex::encode([9u8; 32]);
    let config = parse_server(&["--seed", "42", "--secret-key", &secret_hex]);
    let keypair = config.keypair().expect("secret key takes precedence");

    assert_eq!(*keypair.secret_key, [9u8; 32]);
}

/// Test configuration parsing with defaults
#[test]
fn test_config_defaults() {
    // We can't easily test Config::new() since it parses CLI args,
    // but we can test the struct directly
    let config = Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: Some("test_key".to_string()),
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: Some(1000000),
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
        fee_tolerance_pct: 5,
        dev_mode: false,
    };

    assert_eq!(config.network(), "preprod");
    assert_eq!(config.network_byte(), 0);
    assert_eq!(config.provider_type(), "blockfrost");
    assert_eq!(config.provider_api_key(), "test_key");
    assert_eq!(config.deposit_confirm_depth(), 15);
    assert_eq!(config.deposit_expiration_blocks(), 1440);
    assert_eq!(config.min_deposit_value(), 1000000);
    assert_eq!(config.max_withdrawal_fee(), 2000000);
    assert_eq!(config.max_tx_size(), 16384);
    assert_eq!(config.fee_tolerance_pct(), 5);
}

/// Test mainnet configuration
#[test]
fn test_config_mainnet() {
    let config = Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: "mainnet".to_string(),
        cardano_provider: "maestro".to_string(),
        cardano_api_key: Some("maestro_key".to_string()),
        cardano_provider_url: Some("https://custom.api.com".to_string()),
        cardano_payment_sk: Some("deadbeef".to_string()),
        xnode_peer_registry_file: Some("/tmp/peers.json".to_string()),
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 20,
        deposit_expiration_blocks: 2000,
        min_deposit_value: Some(2000000),
        max_tx_size: 32768,
        max_withdrawal_fee: 3000000,
        fee_tolerance_pct: 10,
        dev_mode: false,
    };

    assert_eq!(config.network(), "mainnet");
    assert_eq!(config.network_byte(), 1);
    assert_eq!(config.provider_type(), "maestro");
    assert_eq!(config.provider_api_key(), "maestro_key");
    assert_eq!(
        config.provider_url(),
        Some("https://custom.api.com".to_string())
    );
    assert_eq!(config.payment_sk(), Some("deadbeef".to_string()));
    assert_eq!(
        config.xnode_peer_registry_file(),
        Some("/tmp/peers.json".to_string())
    );
    assert_eq!(config.deposit_confirm_depth(), 20);
    assert_eq!(config.deposit_expiration_blocks(), 2000);
    assert_eq!(config.min_deposit_value(), 2000000);
    assert_eq!(config.max_tx_size(), 32768);
    assert_eq!(config.fee_tolerance_pct(), 10);
}

/// Test network namespace byte mapping
#[test]
fn test_network_bytes() {
    let cases = [("preprod", 0u8), ("preview", 2u8), ("testnet", 3u8)];
    for (network, expected) in cases {
        let config = Config::Server {
            addr: "0.0.0.0:9999".parse().unwrap(),
            seed: None,
            secret_key: None,
            cardano_network: network.to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: None,
            cardano_provider_url: None,
            cardano_payment_sk: None,
            xnode_peer_registry_file: None,
            xnode_node_id: "node://local".to_string(),
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: None,
            max_tx_size: 16384,
            max_withdrawal_fee: 2000000,
            fee_tolerance_pct: 5,
            dev_mode: false,
        };
        assert_eq!(config.network_byte(), expected, "unexpected network byte for {network}");
    }
}

#[test]
fn test_testnets_have_distinct_network_bytes_for_db_namespacing() {
    let make = |network: &str| Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: network.to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
        fee_tolerance_pct: 5,
        dev_mode: false,
    };

    let preprod = make("preprod").network_byte();
    let preview = make("preview").network_byte();
    let testnet = make("testnet").network_byte();

    assert_ne!(preprod, preview);
    assert_ne!(preprod, testnet);
    assert_ne!(preview, testnet);
}

/// Test default values when optional fields are None
#[test]
fn test_config_default_values() {
    let config = Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
        fee_tolerance_pct: 5,
        dev_mode: false,
    };

    // API key should not silently default to a fake key
    assert_eq!(config.provider_api_key(), "");

    // min_deposit_value should default to 1_000_000
    assert_eq!(config.min_deposit_value(), 1_000_000);

    // payment_sk should be None
    assert!(config.payment_sk().is_none());

    // fee_tolerance_pct should be 5
    assert_eq!(config.fee_tolerance_pct(), 5);
}

/// Test fee tolerance percentage bounds
#[test]
fn test_fee_tolerance_bounds() {
    // Test that values over 100 are clamped
    let config = Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
        fee_tolerance_pct: 150, // Over 100
        dev_mode: false,
    };

    assert_eq!(config.fee_tolerance_pct(), 100);

    // Test zero tolerance
    let config_zero = Config::Server {
        addr: "0.0.0.0:9999".parse().unwrap(),
        seed: None,
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
        fee_tolerance_pct: 0,
        dev_mode: false,
    };

    assert_eq!(config_zero.fee_tolerance_pct(), 0);
}
