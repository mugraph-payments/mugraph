macro_rules! impl_bitset {
    ($size:tt) => {
        paste::paste! {
            #[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash, test_strategy::Arbitrary)]
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
                    debug_assert!(index <= $size);
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
                    debug_assert!(index <= $size);
                    self.0 |= 1 << index;
                }

                #[inline]
                pub fn remove(&mut self, index: [<u $size>]) {
                    debug_assert!(index <= $size);
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
