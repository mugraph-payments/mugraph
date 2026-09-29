use core::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{Deref, DerefMut},
};

use proptest::prelude::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::Error;

pub const POLICY_ID_SIZE: usize = 28;
pub const ASSET_NAME_MAX_SIZE: usize = 32;
pub const ASSET_ID_BYTES_SIZE: usize = POLICY_ID_SIZE + 4 + ASSET_NAME_MAX_SIZE;

#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PolicyId(#[serde(with = "muhex::serde")] pub [u8; POLICY_ID_SIZE]);

impl Arbitrary for PolicyId {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<[u8; POLICY_ID_SIZE]>().prop_map(Self).boxed()
    }
}

impl PolicyId {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; POLICY_ID_SIZE])
    }

    pub fn random<R: RngCore>(rng: &mut R) -> Self {
        let mut output = [0u8; POLICY_ID_SIZE];
        rng.fill_bytes(&mut output);
        Self(output)
    }
}

impl AsRef<[u8; POLICY_ID_SIZE]> for PolicyId {
    #[inline]
    fn as_ref(&self) -> &[u8; POLICY_ID_SIZE] {
        &self.0
    }
}

impl Deref for PolicyId {
    type Target = [u8; POLICY_ID_SIZE];

    #[inline]
    fn deref(&self) -> &[u8; POLICY_ID_SIZE] {
        &self.0
    }
}

impl DerefMut for PolicyId {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8; POLICY_ID_SIZE] {
        &mut self.0
    }
}

impl From<[u8; POLICY_ID_SIZE]> for PolicyId {
    #[inline]
    fn from(value: [u8; POLICY_ID_SIZE]) -> Self {
        Self(value)
    }
}

impl LowerHex for PolicyId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.0), f)
    }
}

impl UpperHex for PolicyId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.0).to_uppercase(), f)
    }
}

impl core::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for PolicyId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetName {
    len: u32,
    bytes: [u8; ASSET_NAME_MAX_SIZE],
}

impl AssetName {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            len: 0,
            bytes: [0u8; ASSET_NAME_MAX_SIZE],
        }
    }

    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > ASSET_NAME_MAX_SIZE {
            return Err(Error::InvalidOperation {
                reason: format!(
                    "asset_name too long: {} bytes (max {})",
                    bytes.len(),
                    ASSET_NAME_MAX_SIZE
                ),
            });
        }

        let mut output = [0u8; ASSET_NAME_MAX_SIZE];
        output[..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            len: bytes.len() as u32,
            bytes: output,
        })
    }

    #[inline]
    pub const fn len_u32(&self) -> u32 {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        &self.bytes[..len]
    }

    #[inline]
    pub const fn as_padded_bytes(&self) -> &[u8; ASSET_NAME_MAX_SIZE] {
        &self.bytes
    }
}

impl Arbitrary for AssetName {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        "[a-zA-Z0-9_]{0,32}"
            .prop_map(|s| {
                Self::new(s.as_bytes()).expect("regex guarantees valid length")
            })
            .boxed()
    }
}

impl Serialize for AssetName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let name = core::str::from_utf8(self.as_bytes())
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for AssetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value.as_bytes()).map_err(serde::de::Error::custom)
    }
}

impl LowerHex for AssetName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.as_bytes()), f)
    }
}

impl UpperHex for AssetName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&muhex::encode(self.as_bytes()).to_uppercase(), f)
    }
}

impl core::fmt::Display for AssetName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

impl core::fmt::Debug for AssetName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("{:x}", self))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    test_strategy::Arbitrary,
    PartialOrd,
    Ord,
)]
pub struct Asset {
    pub policy_id: PolicyId,
    pub asset_name: AssetName,
}

impl Asset {
    pub fn write_bytes(&self, out: &mut [u8]) {
        write_asset_bytes(&self.policy_id, &self.asset_name, out);
    }

    pub fn to_bytes(&self) -> [u8; ASSET_ID_BYTES_SIZE] {
        let mut out = [0u8; ASSET_ID_BYTES_SIZE];
        self.write_bytes(&mut out);
        out
    }

    /// ADA uses the zero policy ID and an empty asset name.
    pub fn is_ada(&self) -> bool {
        self.policy_id == PolicyId::zero() && self.asset_name.is_empty()
    }

    /// The unit string that Cardano providers use: "lovelace" for ADA,
    /// else the policy ID and the asset name in hex.
    pub fn cardano_unit(&self) -> String {
        if self.is_ada() {
            return "lovelace".to_string();
        }

        format!(
            "{}{}",
            muhex::encode(self.policy_id.0),
            muhex::encode(self.asset_name.as_bytes())
        )
    }

    /// The inverse of [`Asset::cardano_unit`].
    pub fn from_cardano_unit(unit: &str) -> Result<Self, Error> {
        if unit == "lovelace" {
            return Ok(Self::default());
        }

        let bytes = muhex::decode(unit).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid asset unit {unit}: {e}"),
        })?;
        if bytes.len() < POLICY_ID_SIZE {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Asset unit {unit} is shorter than a policy ID"
                ),
            });
        }

        let mut policy_id = [0u8; POLICY_ID_SIZE];
        policy_id.copy_from_slice(&bytes[..POLICY_ID_SIZE]);

        Ok(Self {
            policy_id: PolicyId(policy_id),
            asset_name: AssetName::new(&bytes[POLICY_ID_SIZE..])?,
        })
    }
}

#[inline]
pub fn write_asset_bytes(
    policy_id: &PolicyId,
    asset_name: &AssetName,
    out: &mut [u8],
) {
    debug_assert_eq!(out.len(), ASSET_ID_BYTES_SIZE);

    out[..POLICY_ID_SIZE].copy_from_slice(policy_id.as_ref());
    out[POLICY_ID_SIZE..POLICY_ID_SIZE + 4]
        .copy_from_slice(&asset_name.len_u32().to_le_bytes());
    out[POLICY_ID_SIZE + 4..].copy_from_slice(asset_name.as_padded_bytes());
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use test_strategy::proptest;

    use super::*;

    /// Round-trip: AssetName::new(bytes).as_bytes() == bytes for all valid lengths.
    ///
    /// Input generated by construction (0..=32 bytes) — no rejection.
    #[proptest]
    fn prop_asset_name_roundtrip(
        #[strategy(proptest::collection::vec(any::<u8>(), 0..=ASSET_NAME_MAX_SIZE))]
        bytes: Vec<u8>,
    ) {
        let name = AssetName::new(&bytes).unwrap();
        prop_assert_eq!(name.as_bytes(), bytes.as_slice());
        prop_assert_eq!(name.len(), bytes.len());
        prop_assert_eq!(name.is_empty(), bytes.is_empty());
    }

    #[proptest]
    fn prop_cardano_unit_round_trips(asset: Asset) {
        let unit = asset.cardano_unit();
        prop_assert_eq!(Asset::from_cardano_unit(&unit)?, asset);
    }

    #[test]
    fn ada_uses_the_lovelace_unit() {
        let ada = Asset::default();
        assert!(ada.is_ada());
        assert_eq!(ada.cardano_unit(), "lovelace");
        assert_eq!(Asset::from_cardano_unit("lovelace").unwrap(), ada);
    }

    #[test]
    fn cardano_unit_is_policy_then_name_in_hex() {
        let asset = Asset {
            policy_id: PolicyId([0x11; POLICY_ID_SIZE]),
            asset_name: AssetName::new(b"token").unwrap(),
        };
        assert_eq!(
            asset.cardano_unit(),
            format!("{}{}", "11".repeat(POLICY_ID_SIZE), "746f6b656e")
        );
    }

    #[test]
    fn from_cardano_unit_rejects_bad_units() {
        assert!(Asset::from_cardano_unit("").is_err());
        assert!(Asset::from_cardano_unit("zz").is_err());
        assert!(Asset::from_cardano_unit(&"11".repeat(20)).is_err());
    }

    /// Boundary: AssetName::new rejects inputs exceeding ASSET_NAME_MAX_SIZE.
    #[proptest]
    fn prop_asset_name_rejects_oversized(
        #[strategy(proptest::collection::vec(any::<u8>(), (ASSET_NAME_MAX_SIZE + 1)..=64))]
        bytes: Vec<u8>,
    ) {
        prop_assert!(AssetName::new(&bytes).is_err());
    }

    /// AssetName serde round-trip via Asset wrapper.
    #[proptest]
    fn prop_asset_serde_roundtrip(asset: Asset) {
        let json = serde_json::to_string(&asset).unwrap();
        let decoded: Asset = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded, asset);
    }
}
