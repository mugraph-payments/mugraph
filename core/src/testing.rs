use crate::{types::*, G};
use proptest::prelude::*;

pub fn scalar() -> impl Strategy<Value = Scalar> {
    any::<[u8; 32]>().prop_map(Scalar::from_bytes_mod_order)
}

pub fn point() -> impl Strategy<Value = Point> {
    scalar().prop_map(|s| s * *G)
}

pub fn keypair() -> impl Strategy<Value = (SecretKey, PublicKey)> {
    scalar().prop_map(|s| (s, *G * s))
}
