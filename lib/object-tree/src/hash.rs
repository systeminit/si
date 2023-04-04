//! A cryptographic hashing strategy which can help to determine if two arbitrary objects are
//! identical, assumning they can be both deterministically serialized into bytes.
//!
//! # Implementation Notes
//!
//! The current implementation uses the [BLAKE3] hashing function, but this strategy is designed to
//! be opaque, meaning that it might be changed in the future.
//!
//! [BLAKE3]: https://github.com/BLAKE3-team/BLAKE3

use std::{default::Default, fmt, str::FromStr};

use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use thiserror::Error;

/// A cryptographic hash value, computed over an input of bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Hash(blake3::Hash);

impl Hash {
    /// Creates and returns a new [struct@Hash] value, computed from an input of bytes.
    #[must_use]
    pub fn new(input: &[u8]) -> Self {
        Self(blake3::hash(input))
    }

    /// Returns a shortened String representation of the hashed value.
    ///
    /// Note that this value might not be sufficient to determine equality and/or uniqueness
    /// between to objects that have the same shortened string.
    ///
    /// In general, this is used in debugging output to help orient the developer and not presented
    /// to end users.
    #[must_use]
    pub fn short_string(&self) -> String {
        let mut s = self.0.to_string();
        s.truncate(10);
        s
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash::new("".as_bytes())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a blake3 hash string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hash::from_str(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

/// An error when parsing a String representation of a [`struct@Hash`].
#[derive(Debug, Error)]
#[error("failed to parse hash hex string")]
pub struct HashParseError(#[from] blake3::HexError);

impl FromStr for Hash {
    type Err = HashParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::{self, Deserializer, IntoDeserializer};

    #[test]
    fn test_deserialize() {
        let hash = Hash::new(b"white ferrari");
        let hash_string = hash.to_string();
        let deserializer: de::value::StrDeserializer<de::value::Error> =
            hash_string.as_str().into_deserializer();
        let hash_deserialized: Hash = deserializer
            .deserialize_any(HashVisitor)
            .expect("able to deserialize");

        assert_eq!(hash, hash_deserialized);
    }
}
