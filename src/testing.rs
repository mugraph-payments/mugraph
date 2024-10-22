#[macro_export]
macro_rules! test_encode_bytes {
    ($t:ty) => {
        paste::paste! {
            #[test_strategy::proptest]
            fn test_encode_decode_bytes(t: $t) {
                use ::proptest::prelude::*;
                use $crate::{Encode, Decode};
                prop_assert_eq!(<$t>::from_bytes(&t.as_bytes()).unwrap(), t);
            }
        }
    };
}

#[macro_export]
macro_rules! test_encode_fields {
    ($t:ty) => {
        paste::paste! {
            #[::test_strategy::proptest]
            fn test_encode_decode_fields(t: $t) {
                use ::proptest::prelude::*;
                use $crate::{EncodeFields, DecodeFields};

                let fields = t.as_fields();
                prop_assert_eq!(<$t>::from_fields(&fields).unwrap(), t);
            }
        }
    };
}
