use base64::{engine::general_purpose, Engine};
use serde::{self, Deserialize, Deserializer, Serializer};
use sodiumoxide::crypto::box_::SecretKey as BoxSecretKey;

use super::encode_secret_key;

pub fn serialize<S>(box_secret_key: &BoxSecretKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = encode_secret_key(box_secret_key);
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BoxSecretKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let box_buffer = general_purpose::STANDARD_NO_PAD
        .decode(s)
        .map_err(serde::de::Error::custom)?;

    BoxSecretKey::from_slice(&box_buffer)
        .ok_or_else(|| serde::de::Error::custom("cannot deserialize secret key"))
}
