use crate::model::component::{KeyFormat, KeyType};

use serde::{de, Deserialize, Deserializer, Serializer};

pub fn key_type_enum_s<S>(enum_value: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match KeyType::from_i32(*enum_value) {
        Some(real_enum) => serializer.serialize_str(&format!("{}", real_enum)),
        None => serializer.serialize_str("RSA"),
    }
}

pub fn key_type_enum_d<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "RSA" => Ok(KeyType::Rsa as i32),
        "DSA" => Ok(KeyType::Dsa as i32),
        "ECDSA" => Ok(KeyType::Ecdsa as i32),
        "ED25519" => Ok(KeyType::Ed25519 as i32),
        _ => Err(de::Error::custom(
            "Cannot convert from string to key type enum",
        )),
    }
}

pub fn key_format_enum_s<S>(enum_value: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match KeyFormat::from_i32(*enum_value) {
        Some(real_enum) => serializer.serialize_str(&format!("{}", real_enum)),
        None => serializer.serialize_str("RFC4716"),
    }
}

pub fn key_format_enum_d<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "RFC4716" => Ok(KeyFormat::Rfc4716 as i32),
        "PKCS8" => Ok(KeyFormat::Pkcs8 as i32),
        "PEM" => Ok(KeyFormat::Pem as i32),
        _ => Err(de::Error::custom(
            "Cannot convert from string to key format enum",
        )),
    }
}
