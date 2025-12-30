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
        any::<[u8; POLICY_ID_SIZE]>()
            .prop_filter("must not be empty", |x| *x != [0u8; POLICY_ID_SIZE])
            .prop_map(Self)
            .boxed()
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
        proptest::collection::vec(any::<u8>(), 0..=ASSET_NAME_MAX_SIZE)
            .prop_map(|name| Self::new(&name).expect("len already validated"))
            .boxed()
    }
}

impl Serialize for AssetName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for AssetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        Self::new(&bytes).map_err(serde::de::Error::custom)
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
pub struct AssetId {
    pub policy_id: PolicyId,
    pub asset_name: AssetName,
}

impl AssetId {
    pub fn write_bytes(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), ASSET_ID_BYTES_SIZE);

        out[..POLICY_ID_SIZE].copy_from_slice(self.policy_id.as_ref());
        out[POLICY_ID_SIZE..POLICY_ID_SIZE + 4]
            .copy_from_slice(&self.asset_name.len_u32().to_le_bytes());
        out[POLICY_ID_SIZE + 4..]
            .copy_from_slice(self.asset_name.as_padded_bytes());
    }

    pub fn to_bytes(&self) -> [u8; ASSET_ID_BYTES_SIZE] {
        let mut out = [0u8; ASSET_ID_BYTES_SIZE];
        self.write_bytes(&mut out);
        out
    }
}

impl core::fmt::Display for AssetId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self.asset_name.is_empty() {
            true => f.write_fmt(format_args!("{}", self.policy_id)),
            false => f.write_fmt(format_args!(
                "{}.{}",
                self.policy_id, self.asset_name
            )),
        }
    }
}

impl core::fmt::Debug for AssetId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}
