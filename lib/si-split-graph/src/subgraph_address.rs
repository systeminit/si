use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("failed to parse hash hex string")]
pub struct SubGraphAddressParseError(#[from] blake3::HexError);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SubGraphAddress(blake3::Hash);

impl SubGraphAddress {
    #[must_use]
    pub fn new(input: &[u8]) -> Self {
        Self(blake3::hash(input))
    }

    pub fn nil() -> Self {
        Self(blake3::Hash::from_bytes([0; 32]))
    }
}

impl FromStr for SubGraphAddress {
    type Err = SubGraphAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s)?))
    }
}

impl std::fmt::Display for SubGraphAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for SubGraphAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct SubGraphAddressVisitor;

impl Visitor<'_> for SubGraphAddressVisitor {
    type Value = SubGraphAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a blake3 hash string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        SubGraphAddress::from_str(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for SubGraphAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SubGraphAddressVisitor)
    }
}
