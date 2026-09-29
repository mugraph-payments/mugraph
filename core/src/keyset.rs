//! Denomination keys.
//!
//! A delegate signs each note with a key that depends on the note's asset
//! and amount. Thus, a blind signature also fixes the value of the note,
//! although the delegate does not see the note. Each amount must be a
//! power of two, so each asset has 64 keys.

use crate::{
    crypto::hash_to_scalar_with_domain,
    error::{Error, Result},
    types::{
        ASSET_ID_BYTES_SIZE, Asset, Keypair, PublicKey, SecretKey,
        write_asset_bytes,
    },
};

pub const KEYSET_SEP: &[u8] = b"mugraph_v1_keyset";

/// The number of denominations for each asset (2^0 thru 2^63).
pub const DENOMINATIONS: usize = 64;

/// Returns true if `amount` is a valid note amount (a power of two).
pub fn is_denomination(amount: u64) -> bool {
    amount.is_power_of_two()
}

/// Splits `amount` into the powers of two that add up to it, smallest
/// first. Zero gives an empty list.
pub fn split_amount(amount: u64) -> Vec<u64> {
    (0..DENOMINATIONS as u32)
        .map(|d| 1u64 << d)
        .filter(|bit| amount & bit != 0)
        .collect()
}

/// Derives the delegate's signing key for one asset and denomination.
pub fn denomination_keypair(
    master: &SecretKey,
    asset: &Asset,
    amount: u64,
) -> Result<Keypair> {
    if !is_denomination(amount) {
        return Err(Error::InvalidInput {
            reason: format!("Amount {amount} is not a power of two"),
        });
    }

    let mut asset_bytes = [0u8; ASSET_ID_BYTES_SIZE];
    write_asset_bytes(&asset.policy_id, &asset.asset_name, &mut asset_bytes);
    let denomination = [amount.trailing_zeros() as u8];

    let secret_key: SecretKey = hash_to_scalar_with_domain(
        KEYSET_SEP,
        &[master.to_scalar().as_bytes(), &asset_bytes, &denomination],
    )
    .into();

    Ok(Keypair {
        public_key: secret_key.public(),
        secret_key,
    })
}

/// Returns the 64 public keys for `asset`. The key at index `d` signs
/// notes of amount `2^d`.
pub fn keyset(master: &SecretKey, asset: &Asset) -> Vec<PublicKey> {
    (0..DENOMINATIONS as u32)
        .map(|d| {
            denomination_keypair(master, asset, 1 << d)
                .expect("powers of two are denominations")
                .public_key
        })
        .collect()
}

/// Returns the public key for `amount` from a keyset that [`keyset`] made.
pub fn keyset_public_key(keys: &[PublicKey], amount: u64) -> Result<PublicKey> {
    if !is_denomination(amount) {
        return Err(Error::InvalidInput {
            reason: format!("Amount {amount} is not a power of two"),
        });
    }

    keys.get(amount.trailing_zeros() as usize)
        .copied()
        .ok_or_else(|| Error::InvalidInput {
            reason: format!("Keyset has no key for amount {amount}"),
        })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn test_is_denomination_matches_power_of_two(amount: u64) {
        prop_assert_eq!(is_denomination(amount), amount.count_ones() == 1);
    }

    #[test]
    fn test_zero_is_not_a_denomination() {
        assert!(!is_denomination(0));
    }

    #[proptest]
    fn test_split_amount_adds_up(amount: u64) {
        let parts = split_amount(amount);

        prop_assert_eq!(
            parts.iter().map(|&p| p as u128).sum::<u128>(),
            amount as u128
        );
        prop_assert!(parts.iter().all(|&p| is_denomination(p)));
        prop_assert!(parts.windows(2).all(|w| w[0] < w[1]));
    }

    #[proptest]
    fn test_denomination_keypair_rejects_other_amounts(
        master: SecretKey,
        asset: Asset,
        #[filter(!#amount.is_power_of_two())] amount: u64,
    ) {
        prop_assert!(denomination_keypair(&master, &asset, amount).is_err());
    }

    #[proptest]
    fn test_denomination_keys_are_distinct(
        master: SecretKey,
        asset: Asset,
        #[strategy(0u32..64)] a: u32,
        #[strategy(0u32..64)] b: u32,
    ) {
        let ka = denomination_keypair(&master, &asset, 1 << a)?;
        let kb = denomination_keypair(&master, &asset, 1 << b)?;

        prop_assert_eq!(ka == kb, a == b);
    }

    #[proptest]
    fn test_denomination_keys_depend_on_asset(
        master: SecretKey,
        a: Asset,
        b: Asset,
        #[strategy(0u32..64)] d: u32,
    ) {
        let ka = denomination_keypair(&master, &a, 1 << d)?;
        let kb = denomination_keypair(&master, &b, 1 << d)?;

        prop_assert_eq!(ka == kb, a == b);
    }

    #[proptest]
    fn test_denomination_keys_depend_on_master(
        a: SecretKey,
        b: SecretKey,
        asset: Asset,
        #[strategy(0u32..64)] d: u32,
    ) {
        let ka = denomination_keypair(&a, &asset, 1 << d)?;
        let kb = denomination_keypair(&b, &asset, 1 << d)?;

        prop_assert_eq!(ka == kb, a.to_scalar() == b.to_scalar());
    }

    #[proptest]
    fn test_keyset_matches_denomination_keys(
        master: SecretKey,
        asset: Asset,
        #[strategy(0u32..64)] d: u32,
    ) {
        let keys = keyset(&master, &asset);
        let pair = denomination_keypair(&master, &asset, 1 << d)?;

        prop_assert_eq!(keys.len(), DENOMINATIONS);
        prop_assert_eq!(keys[d as usize], pair.public_key);
        prop_assert_eq!(keyset_public_key(&keys, 1 << d)?, pair.public_key);
    }

    #[proptest]
    fn test_keyset_public_key_rejects_other_amounts(
        master: SecretKey,
        asset: Asset,
        #[filter(!#amount.is_power_of_two())] amount: u64,
    ) {
        let keys = keyset(&master, &asset);

        prop_assert!(keyset_public_key(&keys, amount).is_err());
    }
}
