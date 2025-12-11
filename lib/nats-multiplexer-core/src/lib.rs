//! This crate contains shared logic for using the [NATS](https://nats.io) multiplexer crates.

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
    rust_2018_idioms,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use si_data_nats::{
    Message,
    OpenTelemetryContext,
};
use tokio::sync::{
    broadcast,
    oneshot,
};

/// The key for storing and accessing a [`broadcast::Receiver`] within a multiplexer. The key corresponds to the subject
/// (including _terminating_ wildcards) that the client would have otherwise subscribed to.
///
/// Example: if the key is "my.nats.subject.>" and the multiplexer is subscribed to "my.nats.>", the receiver would
/// receive messages such as "my.nats.subject.8675309" and "my.nats.subject.poopy.pants".
pub type MultiplexerKey = String;

/// The request payload that multiplexer client can send to a running multiplexer over a [`oneshot`] channel.
#[derive(Debug)]
pub enum MultiplexerRequest {
    /// Add a receiver for the multiplexer to send to for a given [`key`](MultiplexerKey).
    Add(
        (
            MultiplexerKey,
            oneshot::Sender<broadcast::Receiver<MultiplexerRequestPayload>>,
        ),
    ),
}

/// The payload within a [`MultiplexerRequest`].
#[derive(Debug, Clone)]
pub struct MultiplexerRequestPayload {
    /// The core NATS message found via the subscription to be sent to all multiplexer clients.
    pub nats_message: Message,
    /// An optional [`OpenTelemetryContext`] that can be used for span linking or following a parent span.
    pub otel_ctx: Option<OpenTelemetryContext>,
}
