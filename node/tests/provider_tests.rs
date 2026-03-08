//! Tests for Cardano provider implementations

use mugraph_node::provider::{
    AssetAmount,
    Provider,
    TxSettlementState,
    evaluate_tx_observation,
};
use proptest::prelude::*;

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
    let custom_url = "https://custom.blockfrost.io/api/v0".to_string();
    let provider = Provider::new(
        "blockfrost",
        "test_key".to_string(),
        "preprod".to_string(),
        Some(custom_url.clone()),
    );

    assert!(provider.is_ok());

    match provider.unwrap() {
        Provider::Blockfrost(p) => {
            assert_eq!(p.base_url, custom_url);
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

#[test]
fn test_observation_reports_invalidation_when_missing_after_canonical() {
    let obs = evaluate_tx_observation("tx1", None, 100, 12, 6, true);
    assert_eq!(obs.state, TxSettlementState::Invalidated);
    assert_eq!(obs.confirmations, 0);
}

#[test]
fn test_observation_reports_confirmed_when_target_reached() {
    let obs = evaluate_tx_observation("tx1", Some(90), 101, 12, 6, false);
    assert_eq!(obs.state, TxSettlementState::Confirmed);
    assert!(obs.confirmations >= 12);
}

proptest! {
    #[test]
    fn prop_confirmations_are_monotonic_with_tip(
        block_height in 1u64..=1_000_000,
        tip1 in 1u64..=1_000_000,
        tip2 in 1u64..=1_000_000,
    ) {
        let low_tip = tip1.min(tip2).max(block_height);
        let high_tip = tip1.max(tip2).max(low_tip);

        let a = evaluate_tx_observation("tx1", Some(block_height), low_tip, 12, 6, false);
        let b = evaluate_tx_observation("tx1", Some(block_height), high_tip, 12, 6, false);

        prop_assert!(b.confirmations >= a.confirmations);
    }

    #[test]
    fn prop_missing_tx_state_depends_on_previous_canonical(
        tip_height in 1u64..=10_000_000,
        previously_canonical in any::<bool>(),
    ) {
        let obs = evaluate_tx_observation("tx1", None, tip_height, 12, 6, previously_canonical);
        let expected = if previously_canonical {
            TxSettlementState::Invalidated
        } else {
            TxSettlementState::NotFound
        };

        prop_assert_eq!(obs.state, expected);
        prop_assert_eq!(obs.confirmations, 0);
    }

    #[test]
    fn prop_pre_finality_observation_stays_confirming(
        (block_height, finality_target, confirmations_below) in (1u64..=1_000_000, 2u64..=500)
            .prop_flat_map(|(block_height, finality_target)| {
                (Just(block_height), Just(finality_target), 0u64..finality_target - 1)
            }),
    ) {
        let tip_height = block_height + confirmations_below;
        let obs = evaluate_tx_observation(
            "tx1",
            Some(block_height),
            tip_height,
            finality_target,
            6,
            false,
        );

        prop_assert_eq!(obs.state, TxSettlementState::Confirming);
        prop_assert!(obs.confirmations < finality_target);
    }

    #[test]
    fn prop_observation_is_deterministic_for_same_inputs(
        block_height in proptest::option::of(1u64..=1_000_000),
        tip_height in 1u64..=1_000_000,
        finality_target in 1u64..=500,
        reorg_tolerance in 1u64..=500,
        previously_canonical in any::<bool>(),
    ) {
        let a = evaluate_tx_observation(
            "tx1",
            block_height,
            tip_height,
            finality_target,
            reorg_tolerance,
            previously_canonical,
        );
        let b = evaluate_tx_observation(
            "tx1",
            block_height,
            tip_height,
            finality_target,
            reorg_tolerance,
            previously_canonical,
        );

        prop_assert_eq!(a, b);
    }
}
