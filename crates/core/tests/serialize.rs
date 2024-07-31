macro_rules! generate_serialize_roundtrip_tests {
    ($([$type:ty, $val: expr]),+) => {
        $(
            paste::paste! {
                #[cfg(feature = "std")]
                #[test_strategy::proptest]
                fn [<test_ $type:snake _serialize_roundtrip>](value: $type) {
                    use mugraph_core::SerializeBytes;
                    use proptest::prelude::*;

                    let size = <$type as SerializeBytes>::SIZE;
                    let mut buffer = vec![0u8; size];
                    value.to_slice(&mut buffer);

                    let deserialized = <$type as SerializeBytes>::from_slice(&buffer).unwrap();
                    prop_assert_eq!(value, deserialized);
                    prop_assert_eq!(size, $val);
                }
            }
        )+
    };
}

#[cfg(feature = "std")]
use mugraph_core::*;

type FissionInput = mugraph_core::programs::fission::Input;
type FissionOutput = mugraph_core::programs::fission::Output;
type FusionInput = mugraph_core::programs::fusion::Input;
type FusionOutput = mugraph_core::programs::fusion::Output;

generate_serialize_roundtrip_tests!(
    [u64, 8],
    [Hash, 32],
    [Signature, 64],
    [FissionInput, 144],
    [FissionOutput, 96],
    [FusionInput, 208],
    [FusionOutput, 96],
    [Note, 104],
    [BlindedNote, 72]
);
