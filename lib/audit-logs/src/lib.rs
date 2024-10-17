//! This crate provides centralized logic for working with the audit logs NATS Jetstream stream.

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
        stream::{Config, DiscardPolicy, RetentionPolicy, Stream},
    },
    jetstream,
};
use si_events::audit_log::AuditLog;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

const STREAM_NAME: &str = "AUDIT_LOGS";
const AUDIT_LOG_SUBJECT: &str = "audit_logs.audit_log";
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

type AuditLogsResult<T> = Result<T, AuditLogsError>;

/// A wrapper around the audit logs stream's NATS Jetstream context with helper methods for
/// interacting with the stream.
#[derive(Debug, Clone)]
pub struct AuditLogsWorkQueue {
    context: jetstream::Context,
}

impl AuditLogsWorkQueue {
    /// Create a new instance of audit logs work queue and ensures the underlying stream is
    /// found or created.
    pub async fn get_or_create(context: jetstream::Context) -> AuditLogsResult<Self> {
        // Ensure the stream is created before we start publishing to it.
        let result = Self { context };
        result.stream().await?;
        Ok(result)
    }

    /// Returns a reference to the NATS Jetstream stream name.
    pub fn steam_name(&self) -> &str {
        STREAM_NAME
    }

    /// Publishes an audit log.
    #[instrument(
        name = "audit_logs_work_queue.publish_audit_log",
        level = "info",
        skip_all,
        fields(
            si.workspace.id = workspace_id,
        )
    )]
    pub async fn publish_audit_log(
        &self,
        workspace_id: &str,
        message: &AuditLog,
    ) -> AuditLogsResult<()> {
        self.publish_message_inner(AUDIT_LOG_SUBJECT, workspace_id, message)
            .await
    }

    /// Returns the audit logs stream.
    pub async fn stream(&self) -> AuditLogsResult<Stream> {
        let config = Config {
            name: self.prefixed_stream_name(STREAM_NAME),
            description: Some("Work queue of audit logs".to_string()),
            subjects: vec![self.prefixed_subject(AUDIT_LOG_SUBJECT, ">")],
            allow_direct: true,
            retention: RetentionPolicy::Limits,
            max_age: Duration::from_secs(THIRTY_DAYS_IN_SECONDS),
            // NOTE(nick): need a max message count or max size for this to work.
            // discard: DiscardPolicy::Old,
            ..Default::default()
        };
        Ok(self.context.get_or_create_stream(config).await?)
    }

    /// Provides the audit log subject with an appropriate prefix and suffix.
    pub fn audit_log_subject(&self, suffix: &str) -> String {
        self.prefixed_subject(AUDIT_LOG_SUBJECT, suffix)
    }

    async fn publish_message_inner(
        &self,
        subject: &str,
        parameters: &str,
        message: &impl Serialize,
    ) -> AuditLogsResult<()> {
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
