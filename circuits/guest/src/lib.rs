use mugraph_core::{Hash, Result};
use risc0_zkvm::sha::{Impl, Sha256};

pub fn digest(value: &[u8]) -> Result<Hash> {
    Impl::hash_bytes(&value).as_bytes().try_into()
}

pub fn combine(a: Hash, b: Hash) -> Result<Hash> {
    let mut value = [0u8; 64];

    value[..32].copy_from_slice(&a.0);
    value[32..].copy_from_slice(&b.0);

    digest(&value)
}

pub fn combine3(a: Hash, b: Hash, c: Hash) -> Result<Hash> {
    let mut value = [0u8; 96];

    value[..32].copy_from_slice(&a.0);
    value[32..64].copy_from_slice(&b.0);
    value[64..].copy_from_slice(&c.0);

    digest(&value)
}
