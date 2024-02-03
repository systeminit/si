//! Telemetry support for running HTTP-related services.

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

mod make_span;
mod on_response;
pub mod propagation;

pub use make_span::{HttpMakeSpan, NetworkTransport};
pub use on_response::HttpOnResponse;
