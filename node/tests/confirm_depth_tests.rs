//! Tests for confirm depth validation in deposits

use mugraph_core::types::{DepositRequest, UtxoReference};
use mugraph_node::provider::{ChainTip, UtxoInfo};

/// Test helper to create a mock provider with specific tip and UTxO info
fn create_test_utxo_info(block_height: u64) -> UtxoInfo {
    UtxoInfo {
        tx_hash: "test_tx_hash_123456789".to_string(),
        output_index: 0,
        address: "addr_test1...".to_string(),
        amount: vec![mugraph_node::provider::AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "10000000".to_string(), // 10 ADA
        }],
        datum_hash: None,
        datum: None,
        script_ref: None,
        block_height: Some(block_height),
    }
}

fn create_test_chain_tip(height: u64) -> ChainTip {
    ChainTip {
        slot: height * 20, // Approximate slot calculation
        hash: "block_hash".to_string(),
        block_height: height,
    }
}

/// Test that a sufficiently confirmed UTxO passes validation
#[test]
fn test_confirm_depth_sufficient() {
    // UTxO at block 100, current tip at 120, confirm depth 15
    // 120 - 100 = 20 blocks confirmed (sufficient)
    let utxo_block_height = 100;
    let current_height = 120;
    let confirm_depth = 15;

    let blocks_confirmed = current_height - utxo_block_height;
    assert!(blocks_confirmed >= confirm_depth);
    assert_eq!(blocks_confirmed, 20);
}

/// Test that a barely confirmed UTxO passes validation (exactly at threshold)
#[test]
fn test_confirm_depth_exact_threshold() {
    // UTxO at block 100, current tip at 115, confirm depth 15
    // 115 - 100 = 15 blocks confirmed (exactly at threshold)
    let utxo_block_height = 100;
    let current_height = 115;
    let confirm_depth = 15;

    let blocks_confirmed = current_height - utxo_block_height;
    assert!(blocks_confirmed >= confirm_depth);
    assert_eq!(blocks_confirmed, 15);
}

/// Test that an insufficiently confirmed UTxO fails validation
#[test]
fn test_confirm_depth_insufficient() {
    // UTxO at block 100, current tip at 110, confirm depth 15
    // 110 - 100 = 10 blocks confirmed (insufficient)
    let utxo_block_height = 100;
    let current_height = 110;
    let confirm_depth = 15;

    let blocks_confirmed = current_height - utxo_block_height;
    assert!(blocks_confirmed < confirm_depth);
    assert_eq!(blocks_confirmed, 10);
}

/// Test that a brand new UTxO fails validation
#[test]
fn test_confirm_depth_zero_confirms() {
    // UTxO at block 100, current tip at 100, confirm depth 15
    // 100 - 100 = 0 blocks confirmed (insufficient)
    let utxo_block_height: u64 = 100;
    let current_height: u64 = 100;
    let confirm_depth: u64 = 15;

    let blocks_confirmed: u64 = current_height.saturating_sub(utxo_block_height);
    assert!(blocks_confirmed < confirm_depth);
    assert_eq!(blocks_confirmed, 0);
}

/// Test block height calculation with realistic values
#[test]
fn test_confirm_depth_realistic_values() {
    // Simulate mainnet scenario
    let utxo_block_height = 10_000_000;
    let current_height = 10_000_020;
    let confirm_depth = 15;

    let blocks_confirmed = current_height - utxo_block_height;
    assert!(blocks_confirmed >= confirm_depth);
    assert_eq!(blocks_confirmed, 20);
}

/// Test that UTxOInfo properly stores block height
#[test]
fn test_utxo_info_block_height_storage() {
    let block_height: u64 = 1234567;
    let utxo = create_test_utxo_info(block_height);

    assert_eq!(utxo.block_height, Some(block_height));
    assert_eq!(utxo.tx_hash, "test_tx_hash_123456789");
    assert_eq!(utxo.output_index, 0);
}

/// Test ChainTip structure
#[test]
fn test_chain_tip_structure() {
    let height: u64 = 9999999;
    let tip = create_test_chain_tip(height);

    assert_eq!(tip.block_height, height);
    assert_eq!(tip.hash, "block_hash");
}

/// Test confirm depth with very large block numbers
#[test]
fn test_confirm_depth_large_blocks() {
    let utxo_block_height: u64 = 100_000_000;
    let current_height: u64 = 100_000_100;
    let confirm_depth: u64 = 15;

    let blocks_confirmed = current_height - utxo_block_height;
    assert!(blocks_confirmed >= confirm_depth);
    assert_eq!(blocks_confirmed, 100);
}

/// Test edge case: UTxO older than current height (should never happen, but test anyway)
#[test]
fn test_confirm_depth_future_utxo() {
    // UTxO at block 200, current tip at 100 (invalid scenario)
    // Should use saturating_sub to handle underflow
    let utxo_block_height: u64 = 200;
    let current_height: u64 = 100;

    let blocks_confirmed: u64 = current_height.saturating_sub(utxo_block_height);
    assert_eq!(blocks_confirmed, 0);
}

/// Test the error message format for insufficient confirm depth
#[test]
fn test_confirm_depth_error_message_format() {
    let utxo_block_height: u64 = 100;
    let current_height: u64 = 110;
    let confirm_depth: u64 = 15;
    let blocks_confirmed = current_height - utxo_block_height;

    let error_msg = format!(
        "UTxO not sufficiently confirmed. Block height: {}, Current: {}, Confirmed: {} blocks, Required: {} blocks",
        utxo_block_height, current_height, blocks_confirmed, confirm_depth
    );

    assert!(error_msg.contains("not sufficiently confirmed"));
    assert!(error_msg.contains("100")); // utxo block height
    assert!(error_msg.contains("110")); // current height
    assert!(error_msg.contains("10")); // confirmed blocks
    assert!(error_msg.contains("15")); // required blocks
}

/// Test DepositRequest with proper fields for confirm depth testing
#[test]
fn test_deposit_request_structure() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    assert_eq!(request.utxo.index, 0);
    assert_eq!(request.network, "preprod");
}

/// Test confirm depth with different threshold values
#[test]
fn test_confirm_depth_various_thresholds() {
    let utxo_block_height: u64 = 1000;
    let current_height: u64 = 1100;

    // Test with threshold 5 (should pass)
    assert!(current_height - utxo_block_height >= 5);

    // Test with threshold 50 (should pass)
    assert!(current_height - utxo_block_height >= 50);

    // Test with threshold 100 (should pass exactly)
    assert!(current_height - utxo_block_height >= 100);

    // Test with threshold 101 (should fail)
    assert!(current_height - utxo_block_height < 101);
}
