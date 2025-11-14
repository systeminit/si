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

use audit_logs_stream::{
    AuditLogsStream,
    AuditLogsStreamError,
};
use billing_events::{
    BillingEventsError,
    BillingEventsWorkQueue,
};
use pending_events::{
    PendingEventsError,
    PendingEventsStream,
};
use si_data_nats::{
    NatsClient,
    jetstream,
};
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum JetstreamStreamsError {
    #[error("audit logs stream error: {0}")]
    AuditLogsStream(#[from] AuditLogsStreamError),
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
    #[error("pending events error: {0}")]
    PendingEvents(#[from] PendingEventsError),
}

/// A client-like wrapper around created NATS Jetstream streams' context(s).
#[derive(Clone, Debug)]
pub struct JetstreamStreams {
    /// The audit logs stream.
    audit_logs: AuditLogsStream,
    /// The billing events work queue.
    billing_events: BillingEventsWorkQueue,
    /// The pending events stream.
    pending_events: PendingEventsStream,
}

impl JetstreamStreams {
    /// Gets or creates the underlying streams for [`NatsStreams`].
    pub async fn new(nats_client: NatsClient) -> Result<Self, JetstreamStreamsError> {
        let jetstream_context = jetstream::new(nats_client);
        Ok(Self {
            audit_logs: AuditLogsStream::get_or_create(jetstream_context.clone()).await?,
            billing_events: BillingEventsWorkQueue::get_or_create(jetstream_context.clone())
                .await?,
            pending_events: PendingEventsStream::get_or_create(jetstream_context).await?,
        })
    }

    /// Returns a reference to the audit logs stream.
    pub fn audit_logs(&self) -> &AuditLogsStream {
        &self.audit_logs
    }

    /// Returns a reference to the the billing events work queue.
    pub fn billing_events(&self) -> &BillingEventsWorkQueue {
        &self.billing_events
    }

    /// Returns a reference to the pending events stream.
    pub fn pending_events(&self) -> &PendingEventsStream {
        &self.pending_events
    }
}
