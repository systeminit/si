//! This crate provides the ability to work with the audit logs stream.

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

use serde_json::Error;
use si_data_nats::{
    async_nats::{
        self,
        jetstream::{
            context::{CreateStreamError, PublishError},
            stream::{Config, RetentionPolicy},
        },
    },
    jetstream, Subject,
};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;

const STREAM_NAME: &str = "AUDIT_LOGS";
const STREAM_DESCRIPTION: &str = "Audit logs";
const SUBJECT_PREFIX: &str = "audit.log";
const THIRTY_DAYS_IN_SECONDS: u64 = 30 * 24 * 60 * 60;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLogsStreamError {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] Error),
}

type Result<T> = std::result::Result<T, AuditLogsStreamError>;

/// A wrapper around the audit logs stream's NATS Jetstream context with helper methods for
/// interacting with the stream.
#[derive(Debug, Clone)]
pub struct AuditLogsStream {
    context: jetstream::Context,
}

impl AuditLogsStream {
    /// "Gets" or creates the audit logs stream wrapper with an underlying NATS JetStream stream.
    pub async fn get_or_create(context: jetstream::Context) -> Result<Self> {
        let object = Self { context };
        object.stream().await?;
        Ok(object)
    }

    /// "Gets" or creates the NATS JetStream stream for the audit logs stream wrapper.
    pub async fn stream(&self) -> Result<async_nats::jetstream::stream::Stream> {
        Ok(self
            .context
            .get_or_create_stream(Config {
                name: self.prefixed_stream_name(STREAM_NAME),
                description: Some(STREAM_DESCRIPTION.to_string()),
                subjects: vec![self.prefixed_subject(SUBJECT_PREFIX, ">")],
                retention: RetentionPolicy::Limits,
                max_age: Duration::from_secs(THIRTY_DAYS_IN_SECONDS),
                ..Default::default()
            })
            .await?)
    }

    /// Returns the subject for consuming [`AuditLogs`](AuditLog) for the entire workspace.
    pub fn publishing_subject_for_workspace(&self, workspace_id: WorkspacePk) -> Subject {
        Subject::from(self.prefixed_subject(SUBJECT_PREFIX, &workspace_id.to_string()))
    }

    /// Returns the subject for consuming [`AuditLogs`](AuditLog) for the entire workspace.
    pub fn consuming_subject_for_workspace(&self, workspace_id: WorkspacePk) -> Subject {
        Subject::from(self.prefixed_subject(SUBJECT_PREFIX, &format!("{workspace_id}.>")))
    }

    /// Returns the subject for consuming [`AuditLogs`](AuditLog) for all workspaces.
    pub fn consuming_subject_for_all_workspaces(&self) -> Subject {
        Subject::from(self.prefixed_subject(SUBJECT_PREFIX, ">"))
    }

    /// Returns the subject for publishing and consuming [`AuditLogs`](AuditLog) for a given change set.
    pub fn subject_for_change_set(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Subject {
        Subject::from(
            self.prefixed_subject(SUBJECT_PREFIX, &format!("{workspace_id}.{change_set_id}")),
        )
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
