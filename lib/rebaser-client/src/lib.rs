//! This crate provides the rebaser [`Client`], which is used for communicating with a running
//! rebaser [`Server`](rebaser_server::Server).

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

use futures::StreamExt;
use rebaser_core::{RebaserMessagingConfig, RequestRebaseMessage, SubjectGenerator};
use si_data_nats::jetstream::{Context, JetstreamError};
use si_data_nats::subject::ToSubject;
use si_data_nats::NatsClient;
use telemetry::prelude::error;
use thiserror::Error;
use ulid::Ulid;

// The client does yet need to have its own config, so it uses the messaging config.
pub use rebaser_core::RebaserMessagingConfig as Config;
pub use rebaser_core::ReplyRebaseMessage;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("jetstream error: {0}")]
    Jetstream(#[from] JetstreamError),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("unexpected empty stream when subscribing to subject: {0}")]
    UnexpectedEmptyStream(String),
}

#[allow(missing_docs)]
pub type ClientResult<T> = Result<T, ClientError>;

/// A tenant-scoped client used for communicating with 1:N rebaser servers.
#[derive(Debug)]
pub struct Client {
    jetstream_ctx: Context,
    nats: NatsClient,
    subject_prefix: Option<String>,
    workspace_id: Ulid,
}

impl Client {
    /// Creates a new [`Client`].
    pub fn new(
        nats: NatsClient,
        messaging_config: RebaserMessagingConfig,
        workspace_id: Ulid,
    ) -> Self {
        Self {
            jetstream_ctx: nats.clone().to_jetstream_ctx(),
            nats,
            subject_prefix: messaging_config.subject_prefix().map(ToOwned::to_owned),
            workspace_id,
        }
    }

    /// Publishes a rebase requester to the rebaser stream.
    pub async fn request_rebase(
        &self,
        to_rebase_change_set_id: Ulid,
        onto_workspace_snapshot_id: Ulid,
        onto_vector_clock_id: Ulid,
    ) -> ClientResult<ReplyRebaseMessage> {
        let subject = SubjectGenerator::request(
            self.workspace_id,
            to_rebase_change_set_id,
            self.subject_prefix.as_ref(),
        );

        let serialized_messaged = serde_json::to_vec(&RequestRebaseMessage {
            to_rebase_change_set_id,
            onto_workspace_snapshot_id,
            onto_vector_clock_id,
        })?;

        let reply_subject = self
            .jetstream_ctx
            .publish_with_reply_mailbox_and_immediately_ack(
                &self.nats,
                subject,
                serialized_messaged.into(),
            )
            .await?;

        // NOTE(nick): we may want to add a timeout in the future when waiting for a reply.
        self.wait_for_reply(reply_subject).await
    }

    async fn wait_for_reply(
        &self,
        reply_subject: impl ToSubject,
    ) -> ClientResult<ReplyRebaseMessage> {
        let reply_subject = reply_subject.to_subject();

        let mut subscriber = self.nats.subscribe(reply_subject.clone()).await?;

        // Get the first immediate message (there should only ever be one) and deserialize it.
        let message: ReplyRebaseMessage = if let Some(serialized_message) = subscriber.next().await
        {
            serde_json::from_slice(serialized_message.payload().to_vec().as_slice())?
        } else {
            return Err(ClientError::UnexpectedEmptyStream(
                reply_subject.to_string(),
            ));
        };

        // Attempt to unsubscribe.
        if let Err(err) = subscriber.unsubscribe().await {
            error!(error = ?err, %reply_subject, "error when unsubscribing");
        }

        Ok(message)
    }
}
