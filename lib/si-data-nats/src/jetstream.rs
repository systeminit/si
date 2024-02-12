//! This module contains tools for working with [NATS Jetstream](https://docs.nats.io/nats-concepts/jetstream).

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

use async_nats::jetstream::consumer::StreamErrorKind;
use async_nats::jetstream::context::{CreateStreamErrorKind, GetStreamErrorKind, PublishErrorKind};
use async_nats::jetstream::stream::ConsumerErrorKind;
use telemetry::prelude::*;
use thiserror::Error;

mod consumer;
mod context;

pub use async_nats::jetstream::consumer::pull::Stream;
pub use consumer::Consumer;
pub use context::Context;
pub use context::REPLY_SUBJECT_HEADER_NAME;

/// Re-export of [`async_nats::jetstream::AckKind`].
pub type AckKind = async_nats::jetstream::AckKind;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum JetstreamError {
    #[error("consumer error: {0}")]
    Consumer(#[from] async_nats::error::Error<ConsumerErrorKind>),
    #[error("create stream error: {0}")]
    CreateStream(#[from] async_nats::error::Error<CreateStreamErrorKind>),
    #[error("get stream error: {0}")]
    GetStream(#[from] async_nats::error::Error<GetStreamErrorKind>),
    #[error("invalid subject name for stream: {0}")]
    InvalidSubjectName(String),
    #[error("publish error: {0}")]
    Publish(#[from] async_nats::error::Error<PublishErrorKind>),
    #[error("stream error: {0}")]
    Stream(#[from] async_nats::error::Error<StreamErrorKind>),
}

#[allow(missing_docs)]
pub type JetstreamResult<T> = Result<T, JetstreamError>;
