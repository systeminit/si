//! This crate provides a centralized location for working with the audit logs NATS JetStream stream.

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

use std::time::Duration;

use serde::Serialize;
use serde_json::Error;
use si_data_nats::{
    async_nats::jetstream::{
        context::{CreateStreamError, PublishError},
        stream::{Config, RetentionPolicy},
    },
    jetstream,
};
use si_events::audit_log::AuditLog;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

const STREAM_NAME: &str = "AUDIT_LOGS";
const PUBLISH_SUBJECT: &str = "audit.log";
const THIRTY_DAYS_IN_SECONDS: u64 = 30 * 24 * 60 * 60;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLogsError {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] Error),
}

type Result<T> = std::result::Result<T, AuditLogsError>;

/// A wrapper around the audit logs stream's NATS Jetstream context with helper methods for
/// interacting with the stream.
#[derive(Debug, Clone)]
pub struct AuditLogsWorkQueue {
    context: jetstream::Context,
}

impl AuditLogsWorkQueue {
    /// "Gets" or creates the NATS JetStream stream for the audit logs work queue.
    pub async fn get_or_create(context: jetstream::Context) -> Result<Self> {
        let work_queue = Self { context };
        work_queue
            .context
            .get_or_create_stream(Config {
                name: work_queue.prefixed_stream_name(STREAM_NAME),
                description: Some("Work queue of audit logs".to_string()),
                subjects: vec![work_queue.prefixed_subject(PUBLISH_SUBJECT, ">")],
                retention: RetentionPolicy::Limits,
                max_age: Duration::from_secs(THIRTY_DAYS_IN_SECONDS),
                ..Default::default()
            })
            .await?;
        Ok(work_queue)
    }

    /// Publishes a audit log.
    #[instrument(
        name = "audit_logs_work_queue.publish",
        level = "info",
        skip_all,
        fields(
            si.workspace.id = workspace_id,
        )
    )]
    pub async fn publish(&self, workspace_id: &str, audit_log: &AuditLog) -> Result<()> {
        self.publish_message_inner(PUBLISH_SUBJECT, workspace_id, audit_log)
            .await
    }

    async fn publish_message_inner(
        &self,
        subject: &str,
        parameters: &str,
        message: &impl Serialize,
    ) -> Result<()> {
        let subject = self.prefixed_subject(subject, parameters);
        let ack = self
            .context
            .publish_with_headers(
                subject,
                propagation::empty_injected_headers(),
                serde_json::to_vec(message)?.into(),
            )
            .await?;
        ack.await?;
        Ok(())
    }

    fn prefixed_stream_name(&self, stream_name: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}_{stream_name}"),
            None => stream_name.to_owned(),
        }
    }

    fn prefixed_subject(&self, subject: &str, suffix: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}.{subject}.{suffix}"),
            None => format!("{subject}.{suffix}"),
        }
    }
}
