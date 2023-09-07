//! This crate provides the rebaser [`Client`], which is used for communicating with a running
//! rebaser [`Server`](rebaser_server::Server).

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

mod client;

pub use client::Client;

use si_rabbitmq::{Delivery, RabbitError};
use telemetry::prelude::error;
use thiserror::Error;
use tokio::time::error::Elapsed;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("unexpected empty delivery for stream: {0}")]
    EmptyDelivery(String),
    #[error("empty message contents for delivery: {0:?}")]
    EmptyMessageContentsForDelivery(Delivery),
    #[error("si rabbitmq error: {0}")]
    Rabbit(#[from] RabbitError),
    #[error("rebaser stream for change set not found")]
    RebaserStreamForChangeSetNotFound,
    #[error("hit timeout while waiting for message on reply stream: {0}")]
    ReplyTimeout(Elapsed),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[allow(missing_docs)]
pub type ClientResult<T> = Result<T, ClientError>;
