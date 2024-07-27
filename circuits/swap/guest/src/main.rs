use mugraph_core::Transaction;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{Impl, Sha256};

const MAX_NOTES: usize = 8;
/// There are at max 8 inputs/outputs in a transaction, so there are at most 4 asset types
const MAX_ASSET_TYPES: usize = 4;

fn main() {
    let transaction: Transaction = env::read();

    assert!(transaction.presence > 0, "no inputs and no outputs");
    assert!(transaction.presence & transaction.kinds > 0, "no inputs");
    assert!(transaction.presence ^ transaction.kinds > 0, "no outputs");

    let mut balances = [0u64; MAX_ASSET_TYPES];
    let mut nullifiers = [[0u8; 32]; MAX_NOTES];
    let mut nullifier_count = 0;

    // Process inputs and outputs
    for i in 0..MAX_NOTES {
        if (transaction.presence & (1 << i)) == 0 {
            break; // End of transaction notes
        }

        let (asset_id_index, amount) = transaction.amounts[i];
        let nullifier = transaction.nullifiers[i];

        assert_eq!(
            transaction.presence & (1 << i) == 0,
            nullifier == [0u8; 32],
            "nullifier is not zero for non-present note, or zero for present note"
        );

        let is_output = (transaction.kinds & (1 << i)) != 0;
        if is_output {
            // Output
            let bytes = [
                transaction.asset_ids[asset_id_index as usize].as_ref(),
                amount.to_le_bytes().as_ref(),
                nullifier.as_ref(),
            ]
            .concat();

            env::commit_slice(&Impl::hash_bytes(&bytes).as_bytes());
            balances[asset_id_index as usize] = balances[asset_id_index as usize]
                .checked_sub(amount)
                .expect("Output amount overflow");
        } else {
            // Input
            env::commit_slice(&nullifier);
            balances[asset_id_index as usize] = balances[asset_id_index as usize]
                .checked_add(amount)
                .expect("Input amount overflow");
        }

        assert!(
            !contains_nullifier(&nullifiers, nullifier_count, &nullifier),
            "duplicate nullifier"
        );
        nullifiers[nullifier_count] = nullifier;
        nullifier_count += 1;
    }

    // Check balance
    for balance in balances.iter() {
        assert!(*balance == 0, "unbalanced transaction");
    }
}

fn contains_nullifier(
    nullifiers: &[[u8; 32]; MAX_NOTES],
    count: usize,
    nullifier: &[u8; 32],
) -> bool {
    nullifiers.iter().take(count).any(|n| n == nullifier)
}
