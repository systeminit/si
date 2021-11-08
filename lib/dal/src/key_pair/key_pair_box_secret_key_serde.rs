use super::encode_secret_key;
use serde::{self, Deserialize, Deserializer, Serializer};
use sodiumoxide::crypto::box_::SecretKey as BoxSecretKey;

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
    let box_buffer =
        base64::decode_config(s, base64::STANDARD_NO_PAD).map_err(serde::de::Error::custom)?;
    let pk = BoxSecretKey::from_slice(&box_buffer).ok_or(serde::de::Error::custom(format!(
        "cannot deserialize secret key"
    )));
    pk
}
