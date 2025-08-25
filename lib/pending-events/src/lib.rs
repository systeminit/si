//! This crate provides a centralized location for working with the pending events NATS JetStream stream.

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
use shuttle_core::{
    DESTINATION_SUBJECT_SUFFIX_HEADER_KEY,
    FINAL_MESSAGE_HEADER_KEY,
};
use si_data_nats::{
    HeaderMap,
    Subject,
    async_nats::{
        self,
        jetstream::{
            context::{
                CreateStreamError,
                PublishError,
            },
            stream::{
                Config,
                RetentionPolicy,
            },
        },
    },
    header,
    jetstream,
};
use si_events::{
    ChangeSetId,
    EventSessionId,
    WorkspacePk,
    audit_log::AuditLog,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

const STREAM_NAME: &str = "PENDING_EVENTS";
const STREAM_DESCRIPTION: &str = "Pending events";
const SUBJECT_PREFIX: &str = "pending.event";

const THIRTY_DAYS_IN_SECONDS: u64 = 30 * 24 * 60 * 60;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PendingEventsError {
    #[error("ack publish final message error: {0}")]
    AckPublishFinalMessage(#[source] PublishError),
    #[error("ack publish message error: {0}")]
    AckPublishMessage(#[source] PublishError),
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("publish final message error: {0}")]
    PublishFinalMessage(#[source] PublishError),
    #[error("publish message error: {0}")]
    PublishMessage(#[source] PublishError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] Error),
}

type Result<T> = std::result::Result<T, PendingEventsError>;

/// A wrapper around the pending events stream's NATS Jetstream context with helper methods for
/// interacting with the stream.
#[derive(Debug, Clone)]
pub struct PendingEventsStream {
    context: jetstream::Context,
}

impl PendingEventsStream {
    /// "Gets" or creates the pending events stream wrapper with an underlying NATS JetStream stream.
    pub async fn get_or_create(context: jetstream::Context) -> Result<Self> {
        let object = Self { context };
        object.stream().await?;
        Ok(object)
    }

    /// "Gets" or creates the NATS JetStream stream for the pending events stream wrapper.
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

    /// Publishes a pending audit log.
    #[instrument(
        name = "pending_events_stream.publish_audit_log",
        level = "debug",
        skip_all
    )]
    pub async fn publish_audit_log(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        event_session_id: EventSessionId,
        audit_log: &AuditLog,
        change_set_id_for_destination_subject_suffix: ChangeSetId,
    ) -> Result<()> {
        let mut headers = propagation::empty_injected_headers();
        headers.insert(
            DESTINATION_SUBJECT_SUFFIX_HEADER_KEY,
            change_set_id_for_destination_subject_suffix.to_string(),
        );
        self.publish_message_inner(
            SUBJECT_PREFIX,
            &Self::assemble_audit_log_parameters(
                &workspace_id.to_string(),
                &change_set_id.to_string(),
                event_session_id,
            ),
            Some(headers),
            audit_log,
            false,
        )
        .await
    }

    /// Publishes a pending audit log.
    #[instrument(
        name = "pending_events_stream.publish_audit_log_final_message",
        level = "debug",
        skip_all
    )]
    pub async fn publish_audit_log_final_message(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        event_session_id: EventSessionId,
    ) -> Result<()> {
        let mut headers = propagation::empty_injected_headers();
        headers.insert(FINAL_MESSAGE_HEADER_KEY, "");
        self.publish_message_inner(
            SUBJECT_PREFIX,
            &Self::assemble_audit_log_parameters(
                &workspace_id.to_string(),
                &change_set_id.to_string(),
                event_session_id,
            ),
            Some(headers),
            &serde_json::json!({}),
            true,
        )
        .await
    }

    /// Returns the subject for publishing and consuming [`AuditLogs`](AuditLog).
    pub fn subject_for_audit_log(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        event_session_id: EventSessionId,
    ) -> Subject {
        Subject::from(self.prefixed_subject(
            SUBJECT_PREFIX,
            &Self::assemble_audit_log_parameters(
                &workspace_id.to_string(),
                &change_set_id.to_string(),
                event_session_id,
            ),
        ))
    }

    fn assemble_audit_log_parameters(
        workspace_id: &str,
        change_set_id: &str,
        event_session_id: EventSessionId,
    ) -> String {
        format!("{workspace_id}.{change_set_id}.{event_session_id}.audit_log")
    }

    async fn publish_message_inner(
        &self,
        subject: &str,
        parameters: &str,
        headers: Option<HeaderMap>,
        message: &impl Serialize,
        final_message: bool,
    ) -> Result<()> {
        let self_clone = self.clone();
        let message = serde_json::to_vec(message)?;
        let subject = subject.to_string();
        let parameters = parameters.to_string();
        let publishing_subject = self.prefixed_subject(&subject, &parameters);

        tokio::spawn(async move {
            let maybe_nats_message_id = match headers.as_ref() {
                Some(headers) => headers.get(header::NATS_MESSAGE_ID).cloned(),
                None => None,
            };

            if let Err(err) = self_clone
                .publish_message_inner_fallible(
                    &subject,
                    &parameters,
                    headers,
                    message.into(),
                    final_message,
                )
                .await
            {
                let metadata = self_clone.context.metadata();
                let nats_message_id = maybe_nats_message_id
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                error!(
                    messaging.client_id = metadata.messaging_client_id(),
                    messaging.destination.name = publishing_subject.as_str(),
                    messaging.message.id = nats_message_id,
                    messaging.nats.server.id = metadata.messaging_nats_server_id(),
                    messaging.nats.server.name = metadata.messaging_nats_server_name(),
                    messaging.nats.server.version = metadata.messaging_nats_server_version(),
                    messaging.system = metadata.messaging_system(),
                    messaging.url = metadata.messaging_url(),
                    network.peer.address = metadata.network_peer_address(),
                    network.protocol.name = metadata.network_protocol_name(),
                    network.protocol.version = metadata.network_protocol_version(),
                    network.transport = metadata.network_transport(),
                    server.address = metadata.server_address(),
                    server.port = metadata.server_port(),
                    si.error.message = ?err,
                    si.final_message = final_message,
                    "publishing to pending_events stream failed",
                );
            }
        });

        Ok(())
    }

    async fn publish_message_inner_fallible(
        &self,
        subject: &str,
        parameters: &str,
        headers: Option<HeaderMap>,
        payload: bytes::Bytes,
        final_message: bool,
    ) -> Result<()> {
        let subject = self.prefixed_subject(subject, parameters);
        let publish_result = self
            .context
            .publish_with_headers(
                subject,
                headers.unwrap_or(propagation::empty_injected_headers()),
                payload,
            )
            .await;

        let ack = if final_message {
            publish_result.map_err(PendingEventsError::PublishFinalMessage)?
        } else {
            publish_result.map_err(PendingEventsError::PublishMessage)?
        };

        if final_message {
            ack.await
                .map_err(PendingEventsError::AckPublishFinalMessage)?;
        } else {
            ack.await.map_err(PendingEventsError::AckPublishMessage)?;
        }

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
