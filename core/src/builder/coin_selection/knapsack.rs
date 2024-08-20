use crate::{builder::CoinSelectionStrategy, types::*};

pub struct KnapsackCoinSelection;

impl CoinSelectionStrategy for KnapsackCoinSelection {
    fn select_coins(&self, available: &[Note], target: u64, asset_id: Hash) -> Vec<Note> {
        let mut candidates: Vec<&Note> = available
            .iter()
            .filter(|note| note.asset_id == asset_id)
            .collect();

        // Sort candidates in descending order of amount
        candidates.sort_by(|a, b| b.amount.cmp(&a.amount));

        let mut selected = Vec::new();
        let mut total = 0;

        // First, try to find an exact match
        if let Some(exact_match) = candidates.iter().find(|&&note| note.amount == target) {
            selected.push((*exact_match).clone());
            return selected;
        }

        // If no exact match, use dynamic programming to solve the knapsack problem
        let mut dp = vec![None; (target + 1) as usize];
        dp[0] = Some(Vec::new());

        for note in candidates.iter() {
            for i in (note.amount..=target).rev() {
                if let Some(prev) = dp[(i - note.amount) as usize].clone() {
                    let mut new_selection = prev.clone();
                    new_selection.push(*note);

                    match dp[i as usize] {
                        None => dp[i as usize] = Some(new_selection),
                        Some(ref current) => {
                            if new_selection.len() < current.len() {
                                dp[i as usize] = Some(new_selection);
                            }
                        }
                    }
                }
            }
        }

        // Find the best solution
        match dp[target as usize].as_ref() {
            Some(best_selection) => {
                for note in best_selection {
                    selected.push((*note).clone());
                    total += note.amount;
                }
            }
            None => {
                // If no exact solution found, use a greedy approach
                for note in candidates {
                    if total >= target {
                        break;
                    }
                    selected.push((*note).clone());
                    total += note.amount;
                }
            }
        }

        // Ensure we have enough coins
        if total < target {
            selected.clear();
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use proptest::{collection::vec, prelude::*};
    use test_strategy::proptest;

    use super::*;

    fn create_note(asset_id: Hash, amount: u64) -> Note {
        Note {
            amount,
            delegate: PublicKey::default(),
            asset_id,
            nonce: Hash::default(),
            signature: Signature::default(),
        }
    }

    #[proptest]
    fn test_exact_match(asset_id: Hash, #[strategy(1..1000u64)] amount: u64) {
        let available = vec![create_note(asset_id, amount)];
        let strategy = KnapsackCoinSelection;
        let selected = strategy.select_coins(&available, amount, asset_id);

        prop_assert_eq!(selected.len(), 1);
        prop_assert_eq!(selected[0].amount, amount);
    }

    #[proptest]
    fn test_insufficient_funds(
        asset_id: Hash,
        #[strategy(1..1000u64)] available_amount: u64,
        #[strategy(1..1000u64)] target_amount: u64,
    ) {
        prop_assume!(available_amount < target_amount);

        let available = vec![create_note(asset_id, available_amount)];
        let strategy = KnapsackCoinSelection;
        let selected = strategy.select_coins(&available, target_amount, asset_id);

        prop_assert!(selected.is_empty());
    }

    #[proptest]
    fn test_multiple_notes(
        asset_id: Hash,
        #[strategy(vec(1..100u64, 1..10))] amounts: Vec<u64>,
        #[strategy(1..1000u64)] target: u64,
    ) {
        let available: Vec<Note> = amounts
            .iter()
            .map(|&amount| create_note(asset_id, amount))
            .collect();
        let total_available: u64 = amounts.iter().sum();

        let strategy = KnapsackCoinSelection;
        let selected = strategy.select_coins(&available, target, asset_id);

        if total_available >= target {
            let selected_total: u64 = selected.iter().map(|note| note.amount).sum();
            prop_assert!(selected_total >= target);
            prop_assert!(selected.len() <= available.len());
        } else {
            prop_assert!(selected.is_empty());
        }
    }

    #[proptest]
    fn test_optimal_selection(
        asset_id: Hash,
        #[strategy(vec(1..100u64, 1..10))] amounts: Vec<u64>,
    ) {
        let available: Vec<Note> = amounts
            .iter()
            .map(|&amount| create_note(asset_id, amount))
            .collect();
        let total_available: u64 = amounts.iter().sum();
        let target = total_available / 2;

        let strategy = KnapsackCoinSelection;
        let selected = strategy.select_coins(&available, target, asset_id);

        if !selected.is_empty() {
            let selected_total: u64 = selected.iter().map(|note| note.amount).sum();
            prop_assert!(selected_total >= target);

            // Check if the selection is optimal (no smaller subset sums to the target)
            for i in 0..selected.len() {
                let subset_total: u64 = selected
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, note)| note.amount)
                    .sum();
                prop_assert!(subset_total < target);
            }
        }
    }

    #[proptest]
    fn test_different_asset_ids(
        asset_id1: Hash,
        asset_id2: Hash,
        #[strategy(1..1000u64)] amount1: u64,
        #[strategy(1..1000u64)] amount2: u64,
        #[strategy(1..1000u64)] target: u64,
    ) {
        prop_assume!(asset_id1 != asset_id2);

        let available = vec![
            create_note(asset_id1, amount1),
            create_note(asset_id2, amount2),
        ];
        let strategy = KnapsackCoinSelection;

        let selected1 = strategy.select_coins(&available, target, asset_id1);
        let selected2 = strategy.select_coins(&available, target, asset_id2);

        prop_assert!(selected1.iter().all(|note| note.asset_id == asset_id1));
        prop_assert!(selected2.iter().all(|note| note.asset_id == asset_id2));
    }
}
