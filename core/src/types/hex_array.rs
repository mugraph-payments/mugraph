//! Serde support for fixed-size byte arrays as lowercase hex strings.

use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

pub fn serialize<S, const N: usize>(
    bytes: &[u8; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(bytes))
}

pub fn deserialize<'de, D, const N: usize>(
    deserializer: D,
) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let text = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
    let mut bytes = [0u8; N];
    hex::decode_to_slice(text.as_bytes(), &mut bytes)
        .map_err(D::Error::custom)?;
    Ok(bytes)
}
