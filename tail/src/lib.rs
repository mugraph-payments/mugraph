mod error;
mod types;
mod util;
mod values;

pub mod prelude {
    pub use super::{error::*, types::*, util::*, values::*};
    pub use super::{impl_associate_bytes_types, test_to_bytes, test_to_hex};
}

#[doc(hidden)]
pub mod __dependencies {
    pub use blake3::{ Hash, Hasher };
    pub use criterion;
    pub use itertools;
    pub use paste;
    pub use proptest;
    pub use rand;
    pub use test_strategy;
    pub use thiserror::Error;
}

#[macro_export]
macro_rules! impl_associate_bytes_types {
    ($type:ty) => {
        impl std::hash::Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.to_bytes().hash(state);
            }
        }

        impl FromHex for $type {
            fn from_hex(input: &str) -> Result<Self> {
                let bytes = hex::decode(input)?;
                Self::from_bytes(&bytes)
            }
        }

        impl ToHex for $type {
            fn to_hex(&self) -> String {
                hex::encode(&ToBytes::to_bytes(self))
            }
        }
    };
}

#[macro_export]
macro_rules! test_to_bytes {
    ($type:ty) => {
        $crate::__dependencies::paste::paste! {
            mod [<test_to_bytes_$type:snake>] {
                use std::{ collections::hash_map::DefaultHasher, hash::Hasher };

                use $crate::__dependencies::{
                    proptest::prelude::*,
                    test_strategy,
                };

                use $crate::prelude::*;
                use super::$type;

                test_to_hex!($type);

                #[test]
                fn test_default_is_zero() {
                    assert!(<$type>::default().is_zero());
                }

                #[test_strategy::proptest(fork = false)]
                fn test_is_zero_is_same_as_zero_bytes(item: $type) {
                    prop_assert_eq!(
                        item.is_zero(),
                        item.to_bytes() == <$type>::default().to_bytes()
                    );
                }

                #[test_strategy::proptest(fork = false)]
                fn test_roundtrip(a: $type) {
                    prop_assert_eq!(a.clone(), <$type>::from_bytes(&a.to_bytes())?);
                }

                #[test_strategy::proptest(fork = false)]
                fn test_output_consistency(a: $type) {
                    prop_assert_eq!(a.to_bytes(), <$type>::from_bytes(&a.to_bytes())?.to_bytes());
                }

                #[test_strategy::proptest(fork = false)]
                fn test_is_different_on_different_objects(a: $type, b: $type) {
                    prop_assert_eq!(a == b, a.to_bytes() == b.to_bytes());
                }

                #[test_strategy::proptest(fork = false)]
                fn test_hash_consistency(a: $type, b: $type) {
                    prop_assert_eq!(a == b, a.hash_bytes() == b.hash_bytes());
                }

                #[test_strategy::proptest(fork = false)]
                fn test_std_hash_consistency(a: $type, b: $type) {
                    let mut hasher_a = DefaultHasher::new();
                    hasher_a.write(&a.to_bytes());

                    let mut hasher_b = DefaultHasher::new();
                    hasher_b.write(&b.to_bytes());

                    prop_assert_eq!(a.hash_bytes() == b.hash_bytes(), hasher_a.finish() == hasher_b.finish());
                }
            }
        }
    };
}

#[macro_export]
macro_rules! test_to_hex {
    ($type:ty) => {
        $crate::__dependencies::paste::paste! {
            mod [<test_to_hex_$type:snake>] {
                use $crate::__dependencies::{
                    proptest::prelude::*,
                    test_strategy,
                };

                use $crate::prelude::*;
                use super::$type;

                #[test_strategy::proptest(fork = false)]
                fn test_roundtrip(a: $type) {
                    prop_assert_eq!(a.clone(), <$type>::from_hex(&a.to_hex())?);
                }

                #[test_strategy::proptest(fork = false)]
                fn test_output_consistency(a: $type) {
                    prop_assert_eq!(a.to_hex(), <$type>::from_hex(&a.to_hex())?.to_hex());
                }

                #[test_strategy::proptest(fork = false)]
                fn test_is_different_on_different_objects(a: $type, b: $type) {
                    prop_assert_eq!(a == b, a.to_hex() == b.to_hex());
                }
            }
        }
    };
}
