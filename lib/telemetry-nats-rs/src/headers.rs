//! NATS header names.

use si_data_nats::HeaderName;

/// A correlation ID.
///
/// The conversation ID identifying the conversation to which the message belongs, represented as a
/// string. Sometimes called “Correlation ID”.
pub static CORRELATION_ID: HeaderName = HeaderName::from_static("X-Correlation-ID");
