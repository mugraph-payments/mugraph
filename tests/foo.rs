mod native {
    use ark_bls12_381::{g1::Config, Bls12_381, Fr, G1Projective, G2Affine};
    use ark_ec::{
        hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve},
        pairing::Pairing,
        *,
    };
    use ark_ff::{field_hashers::DefaultFieldHasher, Field};
    use sha2::Sha256;

    fn hash_to_curve(message: &[u8]) -> G1Projective {
        let hasher =
        MapToCurveBasedHasher::<G1Projective, DefaultFieldHasher<Sha256>, WBMap<Config>>::new(&[])
            .unwrap();

        hasher.hash(message).unwrap().into()
    }

    pub fn blind(message: &[u8], r: &Fr) -> G1Projective {
        let h = hash_to_curve(message);
        h * r
    }

    pub fn sign_blinded(sk: &Fr, blinded_message: G1Projective) -> G1Projective {
        blinded_message * sk
    }

    pub fn unblind(blinded_signature: G1Projective, r: &Fr) -> G1Projective {
        let r_inv = r.inverse().unwrap();
        blinded_signature * r_inv
    }

    pub fn verify(pk: &G2Affine, message: &[u8], signature: &G1Projective) -> bool {
        let h = hash_to_curve(message).into_affine();
        let pairing_lhs = Bls12_381::pairing(signature.into_affine(), G2Affine::generator());
        let pairing_rhs = Bls12_381::pairing(h, *pk);
        pairing_lhs == pairing_rhs
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    #[proptest]
    fn test_native_bdhke_workflow(message: Vec<u8>) {
        use ark_bls12_381::{Fr, G2Projective};
        use ark_ec::*;
        use ark_ff::UniformRand;
        use rand::rngs::OsRng;

        use super::native::*;

        // Key Generation
        let mut rng = OsRng;
        let sk = Fr::rand(&mut rng);
        let pk = (G2Projective::generator() * sk).into_affine();

        // User Blinds the Message
        let r = Fr::rand(&mut rng);
        let blinded_message = blind(&message, &r);

        // Signer Signs the Blinded Message
        let blinded_signature = sign_blinded(&sk, blinded_message);

        // User Unblinds the Signature
        let signature = unblind(blinded_signature, &r);

        // Verification
        prop_assert!(verify(&pk, &message, &signature));
    }
}
