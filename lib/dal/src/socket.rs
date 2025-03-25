//! Sockets are the mechanisms to pass and transform data between attributes.

use serde::{Deserialize, Serialize};
use si_pkg::SocketSpecArity;
use strum::{AsRefStr, Display, EnumIter, EnumString};

pub mod connection_annotation;
pub mod debug;
pub mod input;
pub mod output;

// This type goes into the content store so cannot be re-ordered, only extended
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
pub enum SocketKind {
    Frame,
    Standard,
}

// This type goes into the content store so can only be extended, not reordered
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
pub enum SocketArity {
    Many,
    One,
}

impl From<&SocketArity> for SocketSpecArity {
    fn from(value: &SocketArity) -> Self {
        match value {
            SocketArity::One => Self::One,
            SocketArity::Many => Self::Many,
        }
    }
}

impl From<SocketSpecArity> for SocketArity {
    fn from(value: SocketSpecArity) -> Self {
        match value {
            SocketSpecArity::One => Self::One,
            SocketSpecArity::Many => Self::Many,
        }
    }
}
