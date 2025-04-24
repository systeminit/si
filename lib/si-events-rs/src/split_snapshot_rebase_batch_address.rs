use std::{
    fmt,
    str::FromStr,
};

use bytes::BytesMut;
use postgres_types::ToSql;
use serde::{
    Deserialize,
    Serialize,
    de::{
        self,
        Visitor,
    },
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SplitSnapshotRebaseBatchAddress(blake3::Hash);

impl SplitSnapshotRebaseBatchAddress {
    #[must_use]
    pub fn new(input: &[u8]) -> Self {
        Self(blake3::hash(input))
    }

    pub fn nil() -> Self {
        Self(blake3::Hash::from_bytes([0; 32]))
    }
}

#[derive(Debug, Error)]
#[error("failed to parse hash hex string")]
pub struct SplitSnapshotRebaseBatchAddressParseError(#[from] blake3::HexError);

impl FromStr for SplitSnapshotRebaseBatchAddress {
    type Err = SplitSnapshotRebaseBatchAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s)?))
    }
}

impl std::fmt::Display for SplitSnapshotRebaseBatchAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for SplitSnapshotRebaseBatchAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct SplitSnapshotRebaseBatchAddressVisitor;

impl Visitor<'_> for SplitSnapshotRebaseBatchAddressVisitor {
    type Value = SplitSnapshotRebaseBatchAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a blake3 hash string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        SplitSnapshotRebaseBatchAddress::from_str(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for SplitSnapshotRebaseBatchAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SplitSnapshotRebaseBatchAddressVisitor)
    }
}

impl ToSql for SplitSnapshotRebaseBatchAddress {
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

impl<'a> postgres_types::FromSql<'a> for SplitSnapshotRebaseBatchAddress {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let hash_string: String = postgres_types::FromSql::from_sql(ty, raw)?;
        Ok(Self(blake3::Hash::from_str(&hash_string)?))
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty == &postgres_types::Type::TEXT
            || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::TEXT)
    }
}
