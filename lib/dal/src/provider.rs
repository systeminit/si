//! Providers are the mechanisms to pass and transform data between attributes.
//!
//! Some rules on providers:
//! - There are _0 or 1_ implicit [`InternalProviders`](crate::InternalProvider) for every
//!   [`Prop`](crate::Prop) in a [`SchemaVariant`](crate::SchemaVariant) (and [`Schema`](crate::Schema))
//! - There are _0 to N_ explicit [`InternalProviders`](crate::InternalProvider) in a
//!   [`SchemaVariant`](crate::SchemaVariant) (and [`Schema`](crate::Schema))
//! - There are _0 to N_ [`ExternalProviders`](crate::ExternalProvider) in a
//!   [`SchemaVariant`](crate::SchemaVariant) (and [`Schema`](crate::Schema))
//! - When a "connection" is made, we will know the specific [`ExternalProvider`](crate::ExternalProvider)
//!   and explicit [`InternalProvider`](crate::InternalProvider) being "connected"
//!   (or we will at least have enough data to know which providers the user wants to "connect")

use serde::{Deserialize, Serialize};
use si_pkg::SocketSpecArity;
use strum::{AsRefStr, Display, EnumIter, EnumString};

pub mod external;
pub mod internal;

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ProviderKind {
    Frame,
    // NOTE(nick): this used to be "Provider" when the enum was "Socket Kind".
    Standard,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Copy,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ProviderArity {
    Many,
    One,
}

impl From<&ProviderArity> for SocketSpecArity {
    fn from(value: &ProviderArity) -> Self {
        match value {
            ProviderArity::One => Self::One,
            ProviderArity::Many => Self::Many,
        }
    }
}

impl From<SocketSpecArity> for ProviderArity {
    fn from(value: SocketSpecArity) -> Self {
        match value {
            SocketSpecArity::One => Self::One,
            SocketSpecArity::Many => Self::Many,
        }
    }
}
