use blake2::{Blake2b, Digest, digest::consts::U32};
use coset::{CoseSign1Builder, HeaderBuilder, TaggedCborSerializable};
use ed25519_dalek::{Signer, SigningKey};

/// Build a CIP-8 COSE_Sign1 structure for deposit authentication.
///
/// - `signing_key`: the Ed25519 signing key
/// - `payload`: the canonical JSON payload bytes
///
/// Returns the tagged CBOR bytes of the COSE_Sign1 structure.
pub fn build_cip8_signature(
    signing_key: &SigningKey,
    payload: &[u8],
) -> Result<Vec<u8>, String> {
    // Protected header: alg = EdDSA (-8 in COSE)
    let protected = HeaderBuilder::new()
        .algorithm(coset::iana::Algorithm::EdDSA)
        .build();

    // Build the COSE_Sign1 structure
    let sign1 = CoseSign1Builder::new()
        .protected(protected)
        .payload(payload.to_vec())
        .create_signature(b"", |tbs_data| {
            let sig = signing_key.sign(tbs_data);
            sig.to_bytes().to_vec()
        })
        .build();

    sign1
        .to_tagged_vec()
        .map_err(|e| format!("COSE serialization error: {e}"))
}

/// Compute Blake2b-256 hash (used for intent_hash in deposit datums).
pub fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = <Blake2b<U32>>::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Compute Blake2b-224 hash (used for pubkey hashes in deposit datums).
pub fn blake2b_224(data: &[u8]) -> [u8; 28] {
    use blake2::{Blake2b, digest::consts::U28};
    let mut hasher = <Blake2b<U28>>::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 28];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use coset::CoseSign1;
    use ed25519_dalek::Verifier;

    #[test]
    fn cip8_signature_roundtrip() {
        let mut rng_bytes = [0u8; 32];
        rand::Fill::fill(&mut rng_bytes, &mut rand::rng());
        let sk = SigningKey::from_bytes(&rng_bytes);
        let payload = b"test canonical payload";

        let tagged_cbor = build_cip8_signature(&sk, payload).unwrap();
        assert!(!tagged_cbor.is_empty());

        // Deserialize and verify
        let sign1 = CoseSign1::from_tagged_slice(&tagged_cbor).unwrap();
        assert_eq!(sign1.payload.as_deref(), Some(payload.as_slice()));

        // Verify the signature
        let tbs = sign1.tbs_data(b"");
        let sig_bytes: [u8; 64] =
            sign1.signature.as_slice().try_into().unwrap();
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        assert!(sk.verifying_key().verify(&tbs, &sig).is_ok());
    }

    #[test]
    fn cip8_uses_eddsa_algorithm() {
        let mut rng_bytes = [0u8; 32];
        rand::Fill::fill(&mut rng_bytes, &mut rand::rng());
        let sk = SigningKey::from_bytes(&rng_bytes);

        let tagged_cbor = build_cip8_signature(&sk, b"test").unwrap();
        let sign1 = CoseSign1::from_tagged_slice(&tagged_cbor).unwrap();

        let alg = sign1.protected.header.alg.as_ref().unwrap();
        assert_eq!(
            *alg,
            coset::RegisteredLabelWithPrivate::Assigned(
                coset::iana::Algorithm::EdDSA
            )
        );
    }

    #[test]
    fn blake2b_256_produces_32_bytes() {
        let hash = blake2b_256(b"hello");
        assert_eq!(hash.len(), 32);
        assert_eq!(hash, blake2b_256(b"hello"));
        assert_ne!(hash, blake2b_256(b"world"));
    }

    #[test]
    fn blake2b_224_produces_28_bytes() {
        let hash = blake2b_224(b"hello");
        assert_eq!(hash.len(), 28);
        assert_eq!(hash, blake2b_224(b"hello"));
        assert_ne!(hash, blake2b_224(b"world"));
    }
}
