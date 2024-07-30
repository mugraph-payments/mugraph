macro_rules! generate_serialize_roundtrip_tests {
    ($($type:ty),+) => {
        $(
            paste::paste! {
                #[cfg(feature = "std")]
                #[test_strategy::proptest]
                fn [<test_ $type:snake _serialize_roundtrip>](value: $type) {
                    use mugraph_core::SerializeBytes;
                    use proptest::prelude::*;

                    let mut buffer = vec![0u8; <$type as SerializeBytes>::SIZE];
                    value.to_slice(&mut buffer);

                    let deserialized = <$type as SerializeBytes>::from_slice(&buffer).unwrap();
                    prop_assert_eq!(value, deserialized);
                }
            }
        )+
    };
}

#[cfg(feature = "std")]
use mugraph_core::*;

type FissionInput = mugraph_core::contracts::fission::Input;
type FissionOutput = mugraph_core::contracts::fission::Output;
type FusionInput = mugraph_core::contracts::fusion::Input;
type FusionOutput = mugraph_core::contracts::fusion::Output;

generate_serialize_roundtrip_tests!(
    u64,
    Hash,
    Signature,
    FissionInput,
    FissionOutput,
    FusionInput,
    FusionOutput,
    Note,
    BlindedNote
);