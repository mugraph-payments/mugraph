use std::{
    ops::{Add, Sub},
    time::Duration,
};

use chrono::prelude::*;
use proptest::prelude::*;

use crate::prelude::{Result, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Timestamp {
    inner: NaiveDateTime,
}

impl_associate_bytes_types!(Timestamp);

impl Timestamp {
    pub fn now() -> Self {
        Self {
            inner: NaiveDateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap(),
        }
    }

    pub fn from_millis(timestamp: i64) -> Option<Self> {
        NaiveDateTime::from_timestamp_millis(timestamp).map(|inner| Self { inner })
    }

    pub fn as_millis(&self) -> i64 {
        self.inner.timestamp_millis()
    }
}

impl Arbitrary for Timestamp {
    type Parameters = (Self, u16);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((start, delta): Self::Parameters) -> Self::Strategy {
        Just(delta)
            .prop_map(move |delta| Self {
                inner: (start - Duration::from_secs(delta.into())).inner,
            })
            .boxed()
    }

    fn arbitrary() -> Self::Strategy {
        (0..u16::MAX)
            .prop_map(|delta| (Self::now(), delta))
            .prop_flat_map(Self::arbitrary_with)
            .boxed()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner + rhs,
        }
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner - rhs,
        }
    }
}

impl ToBytes for Timestamp {
    type Output = [u8; 6];

    fn to_bytes(&self) -> Self::Output {
        // Assuring we only use 43 bits
        let mask = self.as_millis() & ((1 << 43) - 1);

        // Convert it into a 6-byte array
        mask.to_be_bytes()[2..8].try_into().unwrap()
    }
}

impl FromBytes for Timestamp {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Reconstruct the 43-bit number from the 6-byte array
        let mut timestamp = bytes
            .iter()
            .fold(0i64, |timestamp, byte| (timestamp << 8) | (*byte as i64));

        // Ensure only the 43 least significant bits are used
        timestamp &= (1 << 43) - 1;

        Self::from_millis(timestamp).ok_or({
            Error::FailedDeserialization(format!(
                "failed to get timestamp for `{}`, invalid_format",
                hex::encode(bytes),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::prelude::*;

    test_to_bytes!(Timestamp);

    #[proptest(fork = false)]
    fn test_equality_ignore_anything_smaller_than_milliseconds(a: Timestamp) {
        let b = Timestamp::from_millis(a.as_millis()).unwrap();
        prop_assert_eq!(a, b);
    }
}
