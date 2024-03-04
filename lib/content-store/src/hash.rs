use bytes::BytesMut;
use std::{fmt, str::FromStr};

use postgres_types::ToSql;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use serde_json::Value;

use thiserror::Error;

/// The [`blake3::Hash`] of a given set of contents.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ContentHash(blake3::Hash);

impl ContentHash {
    /// Create a new [`ContentHash`] from a byte array.
    #[must_use]
    pub fn new(input: &[u8]) -> Self {
        Self(blake3::hash(input))
    }

    /// Provide a [`hasher`](ContentHasher) to create [`hashes`](ContentHash).
    pub fn hasher() -> ContentHasher {
        ContentHasher::new()
    }
}

impl From<&Value> for ContentHash {
    fn from(value: &Value) -> Self {
        let input = value.to_string();
        Self::new(input.as_bytes())
    }
}

impl From<&str> for ContentHash {
    fn from(input: &str) -> Self {
        Self::new(input.as_bytes())
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        Self::new("".as_bytes())
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.0)
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct ContentHashVisitor;

impl<'de> Visitor<'de> for ContentHashVisitor {
    type Value = ContentHash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a blake3 hash string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ContentHash::from_str(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ContentHashVisitor)
    }
}

#[derive(Debug, Error)]
#[error("failed to parse hash hex string")]
pub struct ContentHashParseError(#[from] blake3::HexError);

impl FromStr for ContentHash {
    type Err = ContentHashParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s)?))
    }
}

#[derive(Debug, Default)]
pub struct ContentHasher(blake3::Hasher);

impl ContentHasher {
    pub fn new() -> Self {
        ContentHasher(blake3::Hasher::new())
    }

    pub fn update(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    pub fn finalize(&self) -> ContentHash {
        ContentHash(self.0.finalize())
    }
}

impl ToSql for ContentHash {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let self_string = self.to_string();

        self_string.to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        String::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let self_string = self.to_string();
        self_string.to_sql_checked(ty, out)
    }
}
