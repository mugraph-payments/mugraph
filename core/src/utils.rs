macro_rules! impl_bitset {
    ($size:tt) => {
        paste::paste! {
            #[derive(
                Debug,
                Default,
                Clone,
                Copy,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                serde::Serialize,
                serde::Deserialize,
                Hash,
                test_strategy::Arbitrary,
            )]
            #[serde(transparent)]
            #[repr(transparent)]
            pub struct [<BitSet $size>](
                [<u $size>]
            );

            impl [<BitSet $size>] {
                pub const fn new() -> Self {
                    Self(0)
                }

                #[inline]
                pub const fn contains(&self, index: [<u $size>]) -> bool {
                    debug_assert!(index < $size);
                    self.0 & (1 << index) != 0
                }

                #[inline]
                pub const fn to_bytes(&self) -> [u8; $size / 8] {
                    self.0.to_le_bytes()
                }

                #[inline]
                pub const fn count_ones(&self) -> u32 {
                    self.0.count_ones()
                }

                #[inline]
                pub const fn count_zeros(&self) -> u32 {
                    self.0.count_zeros()
                }

                #[inline]
                pub const fn is_empty(&self) -> bool {
                    self.0 == 0
                }

                #[inline]
                pub fn insert(&mut self, index: [<u $size>]) {
                    debug_assert!(index < $size);
                    self.0 |= 1 << index;
                }

                #[inline]
                pub fn remove(&mut self, index: [<u $size>]) {
                    debug_assert!(index < $size);
                    self.0 &= !(1 << index);
                }
            }

            impl From<[u8; $size / 8]> for [<BitSet $size>] {
                fn from(bytes: [u8; $size / 8]) -> Self {
                    Self([<u $size>]::from_le_bytes(bytes))
                }
            }

            impl From<[<BitSet $size>]> for [u8; $size / 8] {
                fn from(bitset: [<BitSet $size>]) -> Self {
                    bitset.to_bytes()
                }
            }
        }
    };
}

impl_bitset!(128);
impl_bitset!(64);
impl_bitset!(32);
impl_bitset!(16);
impl_bitset!(8);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    // --- Round-trip ---

    #[proptest]
    fn prop_bitset32_bytes_roundtrip(bs: BitSet32) {
        let recovered = BitSet32::from(bs.to_bytes());
        prop_assert_eq!(bs, recovered);
    }

    #[proptest]
    fn prop_bitset64_bytes_roundtrip(bs: BitSet64) {
        let recovered = BitSet64::from(bs.to_bytes());
        prop_assert_eq!(bs, recovered);
    }

    #[proptest]
    fn prop_bitset128_bytes_roundtrip(bs: BitSet128) {
        let recovered = BitSet128::from(bs.to_bytes());
        prop_assert_eq!(bs, recovered);
    }

    // --- Algebraic: insert/remove/contains ---

    #[proptest]
    fn prop_bitset32_insert_contains(#[strategy(0u32..32)] idx: u32) {
        let mut bs = BitSet32::new();
        bs.insert(idx);
        prop_assert!(bs.contains(idx));
    }

    #[proptest]
    fn prop_bitset32_remove_not_contains(#[strategy(0u32..32)] idx: u32) {
        let mut bs = BitSet32::new();
        bs.insert(idx);
        bs.remove(idx);
        prop_assert!(!bs.contains(idx));
    }

    #[proptest]
    fn prop_bitset32_empty_after_new() {
        let bs = BitSet32::new();
        prop_assert!(bs.is_empty());
        prop_assert_eq!(bs.count_ones(), 0);
    }

    #[proptest]
    fn prop_bitset32_count_ones_matches_inserts(
        #[strategy(proptest::collection::btree_set(0u32..32, 0..=16))]
        indices: std::collections::BTreeSet<u32>,
    ) {
        let mut bs = BitSet32::new();
        for &idx in &indices {
            bs.insert(idx);
        }
        prop_assert_eq!(bs.count_ones(), indices.len() as u32);
    }

    #[proptest]
    fn prop_bitset32_is_empty_iff_count_zero(bs: BitSet32) {
        prop_assert_eq!(bs.is_empty(), bs.count_ones() == 0);
    }

    #[test]
    #[should_panic]
    fn bitset32_insert_rejects_upper_bound_index() {
        let mut bs = BitSet32::new();
        bs.insert(32);
    }

    #[test]
    #[should_panic]
    fn bitset32_contains_rejects_upper_bound_index() {
        let bs = BitSet32::new();
        let _ = bs.contains(32);
    }

    #[test]
    #[should_panic]
    fn bitset32_remove_rejects_upper_bound_index() {
        let mut bs = BitSet32::new();
        bs.remove(32);
    }
}
