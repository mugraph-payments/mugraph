use crate::crypto::*;
use proptest::prelude::*;

pub fn scalar() -> impl Strategy<Value = Scalar> {
    any::<[u8; 32]>().prop_map(|bytes| Scalar::from_bytes_mod_order(bytes))
}

pub fn point() -> impl Strategy<Value = RistrettoPoint> {
    scalar().prop_map(|s| s * *G)
}

pub fn keypair() -> impl Strategy<Value = (SecretKey, PublicKey)> {
    scalar().prop_map(|s| (s, *G * s))
}
