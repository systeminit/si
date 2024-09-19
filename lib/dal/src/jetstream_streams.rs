//! This module provides the ability to have a wrapper around NATS Jetstream context(s) for already
//! found or created streams.

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

use billing_events::{BillingEventsError, BillingEventsWorkQueue};
use si_data_nats::{jetstream, NatsClient};
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum JetstreamStreamsError {
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
}

/// A client-like wrapper around created NATS Jetstream streams' context(s).
#[derive(Clone, Debug)]
pub struct JetstreamStreams {
    /// The billing events work queue.
    billing_events: BillingEventsWorkQueue,
}

impl JetstreamStreams {
    /// Gets or creates the underlying streams for [`NatsStreams`].
    pub async fn new(nats_client: NatsClient) -> Result<Self, JetstreamStreamsError> {
        Ok(Self {
            billing_events: BillingEventsWorkQueue::get_or_create(jetstream::new(nats_client))
                .await?,
        })
    }

    /// Returns a reference to the the billing events work queue.
    pub fn billing_events(&self) -> &BillingEventsWorkQueue {
        &self.billing_events
    }
}
