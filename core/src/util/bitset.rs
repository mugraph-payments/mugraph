macro_rules! impl_bitset {
    ($size:tt) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
            #[serde(transparent)]
            pub struct [<BitSet $size>](
                [<u $size>]
            );

            impl [<BitSet $size>] {
                pub fn new() -> Self {
                    Self(0)
                }

                pub fn insert(&mut self, index: u8) {
                    assert!(index <= $size);

                    self.0 |= 1 << index;
                }

                pub fn contains(&self, index: u8) -> bool {
                    assert!(index <= $size);

                    self.0 & (1 << index) != 0
                }

                pub fn remove(&mut self, index: u8) {
                    assert!(index <= $size);

                    self.0 &= !(1 << index);
                }

                pub fn to_bytes(&self) -> [u8; $size / 8] {
                    self.0.to_le_bytes()
                }

                pub fn count_ones(&self) -> u32 {
                    self.0.count_ones()
                }

                pub fn count_zeros(&self) -> u32 {
                    self.0.count_zeros()
                }

                pub fn is_empty(&self) -> bool {
                    self.0 == 0
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
