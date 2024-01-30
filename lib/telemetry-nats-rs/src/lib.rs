//! Telemetry support for running NATS-related services.

#![warn(
    clippy::unwrap_in_result,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]

pub mod headers;
mod make_span;
mod propagation;

pub use make_span::NatsMakeSpan;
pub use propagation::{
    empty_injected_headers, extract_opentelemetry_context, inject_headers,
    inject_opentelemetry_context,
};
