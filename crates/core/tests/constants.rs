use mugraph_core::{CHANGE_SEP, HTC_SEP, OUTPUT_SEP};
use sha2::{Digest, Sha256};

#[test]
fn test_const_hash_output_sep() {
    let mut hasher = Sha256::new();
    hasher.update(b"MUGRAPH_OUTPUT");

    assert_eq!(*OUTPUT_SEP, *hasher.finalize());
}

#[test]
fn test_const_hash_change_sep() {
    let mut hasher = Sha256::new();
    hasher.update(b"MUGRAPH_CHANGE");

    assert_eq!(*CHANGE_SEP, *hasher.finalize());
}

#[test]
fn test_const_hash_htc_sep() {
    let mut hasher = Sha256::new();
    hasher.update(b"MUGRAPH_HTC");

    assert_eq!(*HTC_SEP, *hasher.finalize());
}
