use rand_core::{CryptoRng, RngCore};

use crate::{crypto::*, error::Result, types::*};

pub struct BlindedPoint {
    pub factor: Scalar,
    pub point: Point,
}

pub fn blind<R: RngCore + CryptoRng>(rng: &mut R, secret_message: &[u8]) -> BlindedPoint {
    let y = hash_to_curve(secret_message);
    let r = Scalar::random(rng);
    let b_prime = y + (G * r);

    BlindedPoint {
        factor: r,
        point: b_prime,
    }
}

pub fn sign_blinded(secret_key: &SecretKey, blinded_point: &Point) -> Point {
    blinded_point * secret_key.to_scalar()
}

pub fn unblind_signature(
    signed_point: &Point,
    blinding_factor: &Scalar,
    signer_public_key: &PublicKey,
) -> Result<Point> {
    Ok(signed_point - (signer_public_key.to_point()? * blinding_factor))
}

pub fn verify(public_key: &PublicKey, message: &[u8], unblinded_point: Point) -> Result<bool> {
    let y = hash_to_scalar(&[HTC_SEP, message]);
    Ok(y * public_key.to_point()? == unblinded_point)
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use rand::prelude::StdRng;
    use test_strategy::proptest;

    use super::*;
    use crate::{testing::rng, types::Keypair};

    #[proptest]
    fn test_blinding_workflow(#[strategy(rng())] mut rng: StdRng, pair: Keypair, msg: Vec<u8>) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert!(verify(&pair.public_key, &msg, unblinded)?);
    }

    #[proptest]
    fn test_signature_validity_equality(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        a: Vec<u8>,
        b: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &a);

        let sig = sign_blinded(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert_eq!(verify(&pair.public_key, &b, unblinded)?, a == b);
    }

    #[proptest]
    fn test_signature_key_validity(
        #[strategy(rng())] mut rng: StdRng,
        a: Keypair,
        b: Keypair,
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded(&a.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &a.public_key)?;

        prop_assert_eq!(verify(&b.public_key, &msg, unblinded)?, a == b);
    }
}
