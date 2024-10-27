use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT as G, RistrettoPoint, Scalar};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::protocol::{*, circuit::*};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DleqProof {
    pub e: Hash,
    pub s: Hash,
}

impl DleqProof {
    pub fn generate(
        secret_key: SecretKey,
        b_prime: &RistrettoPoint,
        c_prime: &RistrettoPoint,
    ) -> Self {
        let mut rng = OsRng;
        let public_key = secret_key.public();
        let r = Scalar::random(&mut rng);
        let secret_key: Scalar = secret_key.into();

        let r1 = r * G;
        let r2 = r * b_prime;

        let bytes = [
            Hash::new(*r1.compress().as_bytes()).as_fields(),
            Hash::new(*r2.compress().as_bytes()).as_fields(),
            public_key.as_fields(),
            Hash::new(*c_prime.compress().as_bytes()).as_fields(),
        ]
        .concat();

        let e: Hash = PoseidonHash::hash_no_pad(&bytes).into();
        let e: Scalar = e.into();

        let s: Scalar = r + e * secret_key;

        DleqProof {
            e: e.into(),
            s: s.into(),
        }
    }

    pub fn verify(
        &self,
        b_prime: RistrettoPoint,
        c_prime: RistrettoPoint,
        a: RistrettoPoint,
    ) -> bool {
        let e: Scalar = self.e.into();
        let s: Scalar = self.s.into();

        let r1 = s * G - e * a;
        let r2 = s * b_prime - e * c_prime;

        let bytes = [
            Hash::new(*r1.compress().as_bytes()).as_fields(),
            Hash::new(*r2.compress().as_bytes()).as_fields(),
            Hash::new(*a.compress().as_bytes()).as_fields(),
            Hash::new(*c_prime.compress().as_bytes()).as_fields(),
        ]
        .concat();

        let e_prime: Hash = PoseidonHash::hash_no_pad(&bytes).into();

        self.e == e_prime
    }
}
