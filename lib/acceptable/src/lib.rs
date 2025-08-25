//! # Acceptable
//!
//! Traits and behaviors that enable negotiable API types that understand their content type,
//! message type, and version.
//!
//! These message types are meant to be upgradable, meaning that older messages can be understood
//! and acted upon on newer services. Conversely older deployed services are able to recognize
//! incoming messages which are newer that what the service is able to support.
//!
//! If feature-enabled, these API types can determine their deserialization strategy based on
//! metadata external to the message bytes, typically via HTTP or NATS headers (a NATS-based
//! implementation is available via the `nats-headers` feature). Similarily if feature-enabled,
//! these types can also determine how to serialize their content to bytes for transmission.
//!
//! Finally another feature-enabled capability allows API authors to derive several trait
//! implementations, drastically cutting down on the amount of boilerplate when crafting request or
//! response types.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

pub(crate) mod all_versions;
pub(crate) mod container;
pub(crate) mod content_info;
pub(crate) mod error;
pub(crate) mod id;
#[cfg(feature = "deserialize")]
pub(crate) mod negotiate;
pub(crate) mod versioned;

#[cfg(feature = "nats-headers")]
pub use crate::error::HeaderMapParseMessageInfoError;
pub use crate::{
    all_versions::AllVersions,
    container::{
        Container,
        IntoContainer,
        SupportsContainers,
    },
    content_info::{
        ContentInfo,
        ContentType,
        MessageType,
        MessageVersion,
    },
    error::{
        BoxError,
        ContentInfoError,
        UnsupportedDefaultContentTypeError,
        UpgradeError,
    },
    id::RequestId,
    versioned::Versioned,
};
#[cfg(feature = "deserialize")]
pub use crate::{
    container::DeserializeContainer,
    error::{
        DeserializeError,
        NegotiateError,
    },
    negotiate::Negotiate,
};
#[cfg(feature = "serialize")]
pub use crate::{
    container::SerializeContainer,
    error::SerializeError,
};

const CONTENT_TYPE_CBOR: &str = "application/cbor";
const CONTENT_TYPE_JSON: &str = "application/json";

#[cfg(feature = "derive")]
pub use acceptable_macros::*;
