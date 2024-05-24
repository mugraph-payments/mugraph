use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    mem::size_of,
    str::FromStr,
};

use itertools::Itertools;
use proptest::{prelude::*, strategy::BoxedStrategy};

use crate::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct HLC {
    pub timestamp: Timestamp,
    pub pubkey: Pubkey,
    pub counter: u64,
}

impl_associate_bytes_types!(HLC);

impl ToBytes for HLC {
    type Output = [u8; 46];

    fn to_bytes(&self) -> Self::Output {
        if *self == HLC::default() {
            return [0u8; 46];
        }

        let mut bytes = [0u8; 46];

        bytes[..6].copy_from_slice(&self.timestamp.to_bytes());
        bytes[6..(6 + size_of::<u64>())].copy_from_slice(&self.counter.to_be_bytes());
        bytes[(6 + size_of::<u64>())..].copy_from_slice(&self.pubkey.to_bytes());

        bytes
    }
}

impl FromBytes for HLC {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let timestamp = Timestamp::from_bytes(bytes[..6].try_into().unwrap())?;

        let counter =
            u64::from_be_bytes(bytes[6..(6 + size_of::<u64>())].try_into().map_err(|e| {
                Error::FailedDeserialization(format!(
                    "failed to get counter for `{}`, expected {} bytes but got {}",
                    hex::encode(bytes),
                    size_of::<u64>(),
                    e
                ))
            })?);

        let pubkey = Pubkey::from_bytes(&bytes[(6 + size_of::<u64>())..])?;

        Ok(Self {
            timestamp,
            counter,
            pubkey,
        })
    }
}

impl HLC {
    pub fn new(account: &Account, timestamp: Timestamp) -> Self {
        Self {
            timestamp,
            pubkey: account.pubkey(),
            counter: 0,
        }
    }

    /// Increment the state of this clock, with a given local timestamp.
    ///
    /// * If the local timestamp is bigger than our timestamp, update our timestamp and set counter
    /// to zero.
    /// * Otherwise, keep the timestamp unchanged and increase the counter by 1.
    pub fn inc(&mut self, timestamp: &Timestamp) {
        if *timestamp > self.timestamp {
            self.counter = 0;
            self.timestamp = *timestamp;
        } else {
            self.counter += 1;
        }
    }

    /// Merge the state of this event with a newly received one.
    ///
    /// * If the local timestamp is larger than both A and B, use that timestamp and reset counter
    ///   to zero.
    /// * If A and B have the same timestamp, set the counter to `max(a, b) + 1`
    /// * If A is newer than B, keep A timestamp and set the counter to `a + 1`
    /// * Otherwise, use B timestamp and set the counter to `b + 1`
    pub fn recv(&mut self, other: &Self, now: Timestamp) {
        if now > self.timestamp && now > other.timestamp {
            self.timestamp = now;
            self.counter = 0;
        } else if self.timestamp == other.timestamp {
            self.counter = self.counter.max(other.counter) + 1;
        } else if self.timestamp > other.timestamp {
            self.counter += 1;
        } else {
            self.timestamp = other.timestamp;
            self.counter = other.counter + 1;
        }
    }
}

impl PartialEq for HLC {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
            && self.counter == other.counter
            && self.pubkey == other.pubkey
    }
}

impl Eq for HLC {}

impl Debug for HLC {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for HLC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.timestamp.as_millis(),
            self.counter,
            self.pubkey.to_hex(),
        )
    }
}

impl FromStr for HLC {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(':').collect_vec();

        if parts.len() != 3 {
            return Err(Error::FailedDeserialization(format!(
                "clock uid must have 3 parts, got {}",
                parts.len()
            )));
        }

        let timestamp = Timestamp::from_millis(parts[0].parse()?).ok_or(
            Error::FailedDeserialization(format!("failed to parse timestamp: {}", parts[0])),
        )?;
        let count = parts[1].parse()?;
        let pubkey = Pubkey::from_hex(parts[2])?;

        Ok(Self {
            timestamp,
            counter: count,
            pubkey,
        })
    }
}

impl PartialOrd for HLC {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HLC {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.timestamp.cmp(&other.timestamp),
            self.counter.cmp(&other.counter),
            self.pubkey.cmp(&other.pubkey),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => Ordering::Equal,
            (Ordering::Equal, Ordering::Equal, count) => count,
            (Ordering::Equal, acc, _) => acc,
            (timestamp, _, _) => timestamp,
        }
    }
}

impl Arbitrary for HLC {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            any::<Timestamp>(),
            (1..u16::MAX).prop_map(u64::from),
            any::<Pubkey>(),
        )
            .prop_map(|(timestamp, count, pubkey)| Self {
                timestamp,
                counter: count,
                pubkey,
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::prelude::*;

    test_to_bytes!(HLC);

    fn pair(
        delay_a: impl Strategy<Value = u64>,
        delay_b: impl Strategy<Value = u64>,
    ) -> impl Strategy<Value = (HLC, HLC)> {
        let accounts = any::<(Account, Account)>();
        let timestamps = (any::<Timestamp>(), delay_a, delay_b)
            .prop_map(|(ts, da, db)| (ts - Duration::from_secs(da), ts - Duration::from_secs(db)));
        (accounts, timestamps).prop_map(|((a, b), (ta, tb))| (HLC::new(&a, ta), HLC::new(&b, tb)))
    }

    #[proptest(fork = false)]
    fn test_equality_against_a_different_timestamp(a: HLC, b: HLC) {
        prop_assume!(a.timestamp != b.timestamp);
        prop_assert_ne!(a, b);
    }

    #[proptest(fork = false)]
    fn test_equality_against_a_different_counter(a: HLC) {
        let mut b = a;
        b.counter += 1;

        prop_assert_ne!(a, b);
    }

    #[proptest(fork = false)]
    fn test_equality_against_a_different_actor(a: HLC, key: Pubkey) {
        let mut b = a;
        b.pubkey = key;

        prop_assert_ne!(a, b);
    }

    #[proptest(fork = false)]
    fn test_equality(a: HLC, b: HLC) {
        prop_assert_eq!(
            a == b,
            a.timestamp == b.timestamp && a.pubkey == b.pubkey && a.counter == b.counter
        );
    }

    #[proptest(fork = false)]
    fn test_bytes_always_have_same_size(a: HLC, b: HLC) {
        prop_assert_eq!(a.to_bytes().len(), b.to_bytes().len())
    }

    #[proptest(fork = false)]
    fn test_bytes_respect_ordering(a: HLC, b: HLC) {
        prop_assert_eq!(a.to_bytes().cmp(&b.to_bytes()), a.cmp(&b))
    }

    #[proptest(fork = false)]
    fn test_to_string_respect_ordering(a: HLC, b: HLC) {
        prop_assert_eq!(a.to_string().cmp(&b.to_string()), a.cmp(&b))
    }

    #[proptest(fork = false)]
    fn test_to_string_equality(a: HLC, b: HLC) {
        prop_assert_eq!(a == b, a.to_string() == b.to_string());
    }

    #[proptest(fork = false)]
    fn test_to_string_roundtrip(a: HLC) {
        prop_assert_eq!(a.to_string().parse::<HLC>()?, a);
    }

    #[proptest(fork = false)]
    fn test_incrementing_a_clock_makes_it_newer_than_before(op: Timestamp) {
        let mut a = HLC::default();
        let b = a;

        a.inc(&op);

        prop_assert!(a > b);
    }

    #[proptest(fork = false)]
    fn test_ordering_is_preserved_even_under_clock_drift(
        #[strategy((0..u16::MAX).prop_map(u64::from))] delay_a: u64,
        #[strategy((0..u16::MAX).prop_map(u64::from))] delay_b: u64,
        #[strategy(pair(0u64..300, 0u64..300))] clocks: (HLC, HLC),
    ) {
        let (mut a, b) = clocks;

        a.recv(&b, a.timestamp + Duration::from_secs(delay_a));
        a.inc(&(a.timestamp + Duration::from_secs(delay_b)));

        prop_assert!(a > b);
    }

    #[proptest(fork = false)]
    fn test_initialization_with_sets_count_to_zero(account: Account, timestamp: Timestamp) {
        let a = HLC::new(&account, timestamp);
        prop_assert_eq!(a.counter, 0);
    }

    #[proptest(fork = false)]
    fn test_inc_when_timestamp_is_newer_than_ours(
        #[strategy((1..u16::MAX).prop_map(u64::from))] delay: u64,
        mut clock: HLC,
    ) {
        let next = clock.timestamp + Duration::from_secs(delay);

        clock.inc(&next);

        prop_assert_eq!(clock.timestamp, next);
        prop_assert_eq!(clock.counter, 0);
    }

    #[proptest(fork = false)]
    fn test_inc_when_timestamp_is_equal_or_before_ours(
        #[strategy((0..u16::MAX).prop_map(u64::from))] delay: u64,
        mut clock: HLC,
    ) {
        let old_timestamp = clock.timestamp;
        let old_counter = clock.counter;
        let next = clock.timestamp - Duration::from_secs(delay);
        clock.inc(&next);

        prop_assert_eq!(clock.timestamp, old_timestamp);
        prop_assert_eq!(clock.counter, old_counter + 1);
    }

    #[proptest(fork = false)]
    fn test_recv_when_local_timestamp_is_newer(
        #[strategy((1..u16::MAX).prop_map(u64::from))] delay: u64,
        mut a: HLC,
        b: HLC,
    ) {
        let now = a.timestamp.max(b.timestamp) + Duration::from_secs(delay);
        a.recv(&b, now);

        prop_assert_eq!(a.timestamp, now);
        prop_assert_eq!(a.counter, 0);
    }

    #[proptest(fork = false)]
    fn test_recv_when_local_timestamp_is_newer_than_just_one(mut a: HLC, mut b: HLC) {
        b.timestamp = a.timestamp + Duration::from_secs(1);
        a.recv(&b, a.timestamp);

        prop_assert_eq!(a.timestamp, b.timestamp);
        prop_assert_eq!(a.counter, b.counter + 1);
    }

    #[proptest(fork = false)]
    fn test_recv_when_both_timestamps_are_equal(mut a: HLC, b: HLC) {
        let counter = a.counter.max(b.counter);

        a.timestamp = b.timestamp;
        a.recv(&b, b.timestamp);

        prop_assert_eq!(a.timestamp, b.timestamp);
        prop_assert_eq!(a.counter, counter + 1);
    }

    #[proptest(fork = false)]
    fn test_recv_when_our_timestamp_is_newer(
        #[strategy((1..u16::MAX).prop_map(u64::from))] delay: u64,
        mut a: HLC,
        b: HLC,
    ) {
        let counter = a.counter;
        let old_timestamp = b.timestamp;

        a.timestamp = b.timestamp + Duration::from_secs(delay);
        a.recv(&b, b.timestamp);

        prop_assert!(a.timestamp > old_timestamp);
        prop_assert_eq!(a.counter, counter + 1);
    }

    #[proptest(fork = false)]
    fn test_recv_when_their_timestamp_is_newer(mut a: HLC, b: HLC) {
        a.timestamp = b.timestamp - Duration::from_secs(1);
        a.recv(&b, a.timestamp);

        prop_assert_eq!(a.timestamp, b.timestamp);
        prop_assert_eq!(a.counter, b.counter + 1);
    }
}
