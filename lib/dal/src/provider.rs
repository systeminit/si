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

pub mod external;
pub mod internal;
