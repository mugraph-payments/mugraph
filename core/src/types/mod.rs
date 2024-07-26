#[cfg(test)]
use crate::testing::*;
use hex::{decode, encode};
#[cfg(test)]
use proptest::{collection::vec, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(test)]
use test_strategy::Arbitrary;

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::VerifierCircuitData, config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputs,
    },
};

pub mod delegate;

pub use curve25519_dalek::traits::*;

pub type Hash = [u8; 32];
pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;
pub type PublicKey = Point;
pub type SecretKey = Scalar;
pub type CompressedPoint = curve25519_dalek::ristretto::CompressedRistretto;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Signature {
    #[cfg_attr(test, strategy(point()))]
    pub r: Point,
    #[cfg_attr(test, strategy(scalar()))]
    pub s: Scalar,
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let r_bytes = self.r.compress().to_bytes();
        let s_bytes = self.s.to_bytes();

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&r_bytes);
        combined.extend_from_slice(&s_bytes);

        serializer.serialize_str(&encode(combined))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = decode(hex_str).map_err(serde::de::Error::custom)?;

        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }

        let r_bytes: [u8; 32] = bytes[..32].try_into().unwrap();
        let s_bytes: [u8; 32] = bytes[32..].try_into().unwrap();

        let r = CompressedPoint::from_slice(&r_bytes)
            .map_err(|_| serde::de::Error::custom("Invalid r value"))?
            .decompress()
            .ok_or_else(|| serde::de::Error::custom("Invalid r value"))?;
        let s = Scalar::from_canonical_bytes(s_bytes).unwrap();

        Ok(Signature { r, s })
    }
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Note {
    /// The ID for the asset in the Cardano blockchain
    pub asset_id: Hash,

    /// The amount included in this note
    pub amount: u64,

    /// Unblinded signature from the server from this note creation
    ///
    /// Equivalent to C in the protocol, returned by the server after minting or swapping
    /// assets.
    pub signature: Signature,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct UnblindedNote {
    pub asset_id: Hash,
    pub amount: u64,
    pub nonce: Hash,
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    pub data: VerifierCircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
}

impl Serialize for Proof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Compress the proof
        let common = self.data.common.clone();
        let digest = self.data.verifier_only.circuit_digest.clone();

        let compressed_proof = self.proof.clone().compress(&digest, &common).unwrap();

        // Serialize the compressed proof with public inputs
        let bytes = compressed_proof.to_bytes();
        serializer.serialize_str(&hex::encode(bytes))
    }
}

#[cfg(test)]
impl proptest::arbitrary::Arbitrary for Proof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        crate::testing::verified_swap()
            .prop_map(|x| x.proof)
            .boxed()
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Commit {}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Swap {
    #[cfg_attr(test, strategy(vec(any::<Signature>(), 0..=16)))]
    pub inputs: Vec<Signature>,

    /// The blinded secrets to be signed by the delegate.
    ///
    /// Corresponds to B' in the protocol.
    #[cfg_attr(test, strategy(vec(point(), 0..=16)))]
    #[serde(with = "serde_point_vec")]
    pub outputs: Vec<Point>,

    pub proof: Proof,
}

mod serde_point {
    #![allow(dead_code)]

    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(point: &Point, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let compressed = point.compress();
        let bytes = compressed.to_bytes();
        serializer.serialize_str(&encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Point, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = decode(hex_str).map_err(serde::de::Error::custom)?;

        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid point length"));
        }

        let compressed = CompressedPoint::from_slice(&bytes)
            .map_err(|_| serde::de::Error::custom("Invalid compressed point"))?;

        compressed
            .decompress()
            .ok_or_else(|| serde::de::Error::custom("Invalid point"))
    }
}

mod serde_point_vec {
    #![allow(dead_code)]

    use super::*;
    use serde::{
        de::{IntoDeserializer, SeqAccess},
        ser::SerializeSeq,
        Deserializer, Serializer,
    };

    pub fn serialize<S>(points: &Vec<Point>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(points.len()))?;
        for point in points {
            let compressed = point.compress();
            let bytes = compressed.to_bytes();
            let hex_str = encode(bytes);
            seq.serialize_element(&hex_str)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Point>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PointVecVisitor;

        impl<'de> serde::de::Visitor<'de> for PointVecVisitor {
            type Value = Vec<Point>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of Points")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut points = Vec::new();
                while let Some(hex_str) = seq.next_element::<String>()? {
                    let point = serde_point::deserialize(hex_str.into_deserializer())?;
                    points.push(point);
                }
                Ok(points)
            }
        }

        deserializer.deserialize_seq(PointVecVisitor)
    }
}
