//! This crate provides [`MultiplexerClient`], which is used to get receivers for a given [NATS](https://nats.io)
//! subject that a running multiplexer is subscribed to.
//!
//! If desired, you can get a stream implementation for a receiver from [`tokio`], such as
//! [`BroadcastStream`](https://docs.rs/tokio-stream/latest/tokio_stream/wrappers/struct.BroadcastStream.html).

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

use std::fmt::Debug;

use nats_multiplexer_core::MultiplexerRequest;
use si_data_nats::subject::ToSubject;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    broadcast,
    mpsc,
    oneshot,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum MultiplexerClientError {
    #[error("oneshot recv error: {0}")]
    OneshotRecv(#[from] oneshot::error::RecvError),
    #[error("send request error: {0}")]
    SendRequest(#[from] mpsc::error::SendError<MultiplexerRequest>),
}

#[allow(missing_docs)]
pub type MultiplexerClientResult<T> = Result<T, MultiplexerClientError>;

/// Re-export the payload that the client will use.
pub use nats_multiplexer_core::MultiplexerRequestPayload;

/// The client used for getting receivers from a running multiplexer.
///
/// **Note: there can only be one client per multiplexer.**
#[derive(Debug)]
pub struct MultiplexerClient {
    // NOTE(nick,fletcher): might make sense later to make these bounded. Unbounded is fine (at the time
    // of writing) because we have a single client per multiplexer.
    tx: mpsc::UnboundedSender<MultiplexerRequest>,
}

impl MultiplexerClient {
    /// Creates a new client using a sender provided during multiplexer creation.
    pub fn new(tx: mpsc::UnboundedSender<MultiplexerRequest>) -> Self {
        Self { tx }
    }

    /// Get a receiver for a given subject. The subject can using wildcards, but they must be _terminating_ wildcards
    /// (e.g. "my.subject.>" or "my.subject.*").
    pub async fn receiver(
        &self,
        subject: impl ToSubject,
    ) -> MultiplexerClientResult<broadcast::Receiver<MultiplexerRequestPayload>> {
        let (reply_tx, reply_rx) =
            oneshot::channel::<broadcast::Receiver<MultiplexerRequestPayload>>();

        // We convert to a subject and then to a string because we need to ensure that it is a valid subject.
        self.tx.send(MultiplexerRequest::Add((
            subject.to_subject().to_string(),
            reply_tx,
        )))?;
        let receiver = reply_rx.await?;
        Ok(receiver)
    }
}
