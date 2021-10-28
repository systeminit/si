use super::encode_public_key;
use serde::{self, Deserialize, Deserializer, Serializer};
use sodiumoxide::crypto::box_::PublicKey as BoxPublicKey;

pub fn serialize<S>(box_public_key: &BoxPublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = encode_public_key(box_public_key);
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BoxPublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let box_buffer =
        base64::decode_config(s, base64::STANDARD_NO_PAD).map_err(serde::de::Error::custom)?;
    let pk = BoxPublicKey::from_slice(&box_buffer).ok_or(serde::de::Error::custom(format!(
        "cannot deserialize public key"
    )));
    pk
}
