//! Tests for Cardano provider implementations

use mugraph_node::provider::{AssetAmount, ChainTip, ProtocolParams, Provider, UtxoInfo};

/// Test Blockfrost provider creation
#[test]
fn test_blockfrost_provider_creation() {
    let provider = Provider::new(
        "blockfrost",
        "test_api_key".to_string(),
        "preprod".to_string(),
        None,
    );

    assert!(provider.is_ok());

    // Check that default URL is used
    match provider.unwrap() {
        Provider::Blockfrost(p) => {
            assert!(p.base_url.contains("blockfrost"));
            assert!(p.base_url.contains("preprod"));
        }
        _ => panic!("Expected Blockfrost provider"),
    }
}

/// Test Maestro provider creation
#[test]
fn test_maestro_provider_creation() {
    let provider = Provider::new(
        "maestro",
        "test_api_key".to_string(),
        "mainnet".to_string(),
        None,
    );

    assert!(provider.is_ok());

    match provider.unwrap() {
        Provider::Maestro(p) => {
            assert!(p.base_url.contains("maestro"));
        }
        _ => panic!("Expected Maestro provider"),
    }
}

/// Test provider with custom URL
#[test]
fn test_provider_custom_url() {
    let custom_url = Some("https://custom.blockfrost.io/api/v0".to_string());
    let provider = Provider::new(
        "blockfrost",
        "test_key".to_string(),
        "preprod".to_string(),
        custom_url.clone(),
    );

    assert!(provider.is_ok());

    match provider.unwrap() {
        Provider::Blockfrost(p) => {
            assert_eq!(p.base_url, custom_url.unwrap());
        }
        _ => panic!("Expected Blockfrost provider"),
    }
}

/// Test provider with different networks
#[test]
fn test_provider_networks() {
    let networks = vec!["mainnet", "preprod", "preview"];

    for network in &networks {
        let provider = Provider::new(
            "blockfrost",
            "test_key".to_string(),
            network.to_string(),
            None,
        );

        assert!(
            provider.is_ok(),
            "Failed to create provider for {}",
            network
        );

        match provider.unwrap() {
            Provider::Blockfrost(p) => {
                assert!(
                    p.base_url.contains(network),
                    "URL should contain {}",
                    network
                );
            }
            _ => panic!("Expected Blockfrost provider"),
        }
    }
}

/// Test invalid provider type
#[test]
fn test_invalid_provider_type() {
    let result = Provider::new(
        "invalid_provider",
        "test_key".to_string(),
        "preprod".to_string(),
        None,
    );

    assert!(result.is_err());
}

/// Test UtxoInfo structure
#[test]
fn test_utxo_info_structure() {
    let utxo = UtxoInfo {
        tx_hash: "deadbeef".to_string(),
        output_index: 5,
        address: "addr_test1...".to_string(),
        amount: vec![
            AssetAmount {
                unit: "lovelace".to_string(),
                quantity: "1000000".to_string(),
            },
            AssetAmount {
                unit: "deadbeef.my_token".to_string(),
                quantity: "100".to_string(),
            },
        ],
        datum_hash: Some("datum_hash".to_string()),
        datum: None,
        script_ref: None,
        block_height: Some(67890),
    };

    assert_eq!(utxo.tx_hash, "deadbeef");
    assert_eq!(utxo.output_index, 5);
    assert_eq!(utxo.amount.len(), 2);
    assert_eq!(utxo.amount[0].unit, "lovelace");
    assert_eq!(utxo.amount[1].quantity, "100");
}

/// Test AssetAmount structure
#[test]
fn test_asset_amount_structure() {
    let asset = AssetAmount {
        unit: "lovelace".to_string(),
        quantity: "5000000".to_string(),
    };

    assert_eq!(asset.unit, "lovelace");
    assert_eq!(asset.quantity, "5000000");

    // Serialization
    let json = serde_json::to_string(&asset).unwrap();
    assert!(json.contains("lovelace"));
    assert!(json.contains("5000000"));

    // Deserialization
    let decoded: AssetAmount = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.unit, "lovelace");
    assert_eq!(decoded.quantity, "5000000");
}

/// Test ChainTip structure
#[test]
fn test_chain_tip_structure() {
    let tip = ChainTip {
        slot: 50000000,
        hash: "block_hash".to_string(),
        block_height: 1000000,
    };

    assert_eq!(tip.slot, 50000000);
    assert_eq!(tip.block_height, 1000000);
    assert_eq!(tip.hash, "block_hash");
}

/// Test ProtocolParams structure
#[test]
fn test_protocol_params_structure() {
    let params = ProtocolParams {
        min_fee_a: 44,
        min_fee_b: 155381,
        max_tx_size: 16384,
        max_val_size: 5000,
        key_deposit: 2000000,
        pool_deposit: 500000000,
        price_mem: 0.0577,
        price_step: 0.0000721,
        max_tx_ex_mem: 14000000,
        max_tx_ex_steps: 10000000000,
        coins_per_utxo_byte: 4310,
    };

    assert_eq!(params.min_fee_a, 44);
    assert_eq!(params.max_tx_size, 16384);
    assert_eq!(params.coins_per_utxo_byte, 4310);

    // Serialization
    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("44"));
    assert!(json.contains("16384"));
}

/// Test provider clone
#[test]
fn test_provider_clone() {
    let provider = Provider::new(
        "blockfrost",
        "test_key".to_string(),
        "preprod".to_string(),
        None,
    )
    .unwrap();

    let cloned = provider.clone();
    // Both should be valid
    match (provider, cloned) {
        (Provider::Blockfrost(p1), Provider::Blockfrost(p2)) => {
            assert_eq!(p1.api_key, p2.api_key);
            assert_eq!(p1.network, p2.network);
        }
        _ => panic!("Expected Blockfrost providers"),
    }
}
