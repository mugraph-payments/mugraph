//! Tests for node configuration

use mugraph_node::config::Config;

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
