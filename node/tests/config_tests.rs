//! Tests for node configuration

use std::net::SocketAddr;

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
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: Some(1000000),
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
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
        deposit_confirm_depth: 20,
        deposit_expiration_blocks: 2000,
        min_deposit_value: Some(2000000),
        max_tx_size: 32768,
        max_withdrawal_fee: 3000000,
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
    assert_eq!(config.deposit_confirm_depth(), 20);
    assert_eq!(config.deposit_expiration_blocks(), 2000);
    assert_eq!(config.min_deposit_value(), 2000000);
    assert_eq!(config.max_tx_size(), 32768);
}

/// Test network byte for different networks
#[test]
fn test_network_bytes() {
    let testnets = vec!["preprod", "preview", "testnet"];
    for network in testnets {
        let config = Config::Server {
            addr: "0.0.0.0:9999".parse().unwrap(),
            seed: None,
            secret_key: None,
            cardano_network: network.to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: None,
            cardano_provider_url: None,
            cardano_payment_sk: None,
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: None,
            max_tx_size: 16384,
            max_withdrawal_fee: 2000000,
        };
        assert_eq!(
            config.network_byte(),
            0,
            "{} should have network_byte 0",
            network
        );
    }
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
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2000000,
    };

    // API key should default to "test_key"
    assert_eq!(config.provider_api_key(), "test_key");

    // min_deposit_value should default to 1_000_000
    assert_eq!(config.min_deposit_value(), 1_000_000);

    // payment_sk should be None
    assert!(config.payment_sk().is_none());
}
