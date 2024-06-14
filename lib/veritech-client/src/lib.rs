use futures::{StreamExt, TryStreamExt};
use nats_subscriber::{Subscriber, SubscriberError};
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::{jetstream, NatsClient, Subject};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use veritech_core::{
    reply_mailbox_for_output, reply_mailbox_for_result, FINAL_MESSAGE_HEADER_KEY,
    NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX, NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX,
    NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX, NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX,
    REPLY_INBOX_HEADER_NAME,
};

pub use cyclone_core::{
    ActionRunRequest, ActionRunResultSuccess, BeforeFunction, ComponentKind, ComponentView,
    FunctionResult, FunctionResultFailure, FunctionResultFailureErrorKind, KillExecutionRequest,
    OutputStream, ResolverFunctionComponent, ResolverFunctionRequest, ResolverFunctionResponseType,
    ResolverFunctionResultSuccess, ResourceStatus, SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess, SensitiveContainer, ValidationRequest,
    ValidationResultSuccess,
};
pub use veritech_core::{encrypt_value_tree, VeritechValueEncryptError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no function result from cyclone; bug!")]
    NoResult,
    #[error("unable to publish message: {0:?}")]
    PublishingFailed(si_data_nats::Message),
    #[error("root connection closed")]
    RootConnectionClosed,
    #[error("subscriber error: {0}")]
    Subscriber(#[from] SubscriberError),
    #[error(transparent)]
    Transport(Box<dyn std::error::Error + Sync + Send + 'static>),
}

pub type ClientResult<T> = Result<T, ClientError>;

/// This _private_ enum helps dictate what NATS technology should be used in communicating with veritech.
enum RequestMode {
    /// Publish messages using core NATS to communicate with veritech.
    Core,
    /// Publish messages using NATS Jetstream to communicate with veritech.
    Jetstream,
}

#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
    context: jetstream::Context,
}

impl Client {
    pub fn new(nats: NatsClient) -> Self {
        let context = jetstream::new(nats.clone());
        Self { nats, context }
    }

    fn nats_subject_prefix(&self) -> Option<&str> {
        self.nats.metadata().subject_prefix()
    }

    #[instrument(
        name = "veritech_client.execute_action_run",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = change_set_id,
            si.workspace.id = workspace_id,
        ),
    )]
    pub async fn execute_action_run(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ActionRunRequest,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<ActionRunResultSuccess>> {
        self.execute_request(
            veritech_core::subject::veritech_request(
                self.nats_subject_prefix(),
                workspace_id,
                change_set_id,
                NATS_ACTION_RUN_DEFAULT_SUBJECT_SUFFIX,
            ),
            Some(output_tx),
            request,
            RequestMode::Jetstream,
        )
        .await
    }

    #[instrument(
        name = "veritech_client.execute_resolver_function",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = change_set_id,
            si.workspace.id = workspace_id,
        ),
    )]
    pub async fn execute_resolver_function(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            veritech_core::subject::veritech_request(
                self.nats_subject_prefix(),
                workspace_id,
                change_set_id,
                NATS_RESOLVER_FUNCTION_DEFAULT_SUBJECT_SUFFIX,
            ),
            Some(output_tx),
            request,
            RequestMode::Jetstream,
        )
        .await
    }

    #[instrument(
        name = "veritech_client.execute_schema_variant_definition",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = change_set_id,
            si.workspace.id = workspace_id,
        ),
    )]
    pub async fn execute_schema_variant_definition(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &SchemaVariantDefinitionRequest,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<SchemaVariantDefinitionResultSuccess>> {
        self.execute_request(
            veritech_core::subject::veritech_request(
                self.nats_subject_prefix(),
                workspace_id,
                change_set_id,
                NATS_SCHEMA_VARIANT_DEFINITION_DEFAULT_SUBJECT_SUFFIX,
            ),
            Some(output_tx),
            request,
            RequestMode::Jetstream,
        )
        .await
    }

    #[instrument(
        name = "veritech_client.execute_validation",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = change_set_id,
            si.workspace.id = workspace_id,
        ),
    )]
    pub async fn execute_validation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ValidationRequest,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<ValidationResultSuccess>> {
        self.execute_request(
            veritech_core::subject::veritech_request(
                self.nats_subject_prefix(),
                workspace_id,
                change_set_id,
                NATS_VALIDATION_DEFAULT_SUBJECT_SUFFIX,
            ),
            Some(output_tx),
            request,
            RequestMode::Jetstream,
        )
        .await
    }

    #[instrument(
        name = "veritech_client.kill_execution",
        level = "info",
        skip_all,
        fields()
    )]
    pub async fn kill_execution(
        &self,
        request: &KillExecutionRequest,
    ) -> ClientResult<FunctionResult<()>> {
        self.execute_request(
            veritech_core::subject::veritech_kill_request(self.nats_subject_prefix()),
            None,
            request,
            RequestMode::Core,
        )
        .await
    }

    async fn execute_request<R, S>(
        &self,
        subject: Subject,
        output_tx: Option<mpsc::Sender<OutputStream>>,
        request: &R,
        request_mode: RequestMode,
    ) -> ClientResult<FunctionResult<S>>
    where
        R: Serialize,
        S: DeserializeOwned,
    {
        let msg = serde_json::to_vec(request).map_err(ClientError::JSONSerialize)?;
        let reply_mailbox_root = self.nats.new_inbox();

        // Construct a subscriber stream for the result
        let result_subscriber_subject = reply_mailbox_for_result(&reply_mailbox_root);
        trace!(
            messaging.destination = &result_subscriber_subject.as_str(),
            "subscribing for result messages"
        );
        let mut result_subscriber: Subscriber<FunctionResult<S>> =
            Subscriber::create(result_subscriber_subject)
                .final_message_header_key(FINAL_MESSAGE_HEADER_KEY)
                .start(&self.nats)
                .await?;

        // Construct a subscriber stream for output messages
        let output_subscriber_subject = reply_mailbox_for_output(&reply_mailbox_root);
        trace!(
            messaging.destination = &output_subscriber_subject.as_str(),
            "subscribing for output messages"
        );
        let output_subscriber = Subscriber::create(output_subscriber_subject)
            .final_message_header_key(FINAL_MESSAGE_HEADER_KEY)
            .start(&self.nats)
            .await?;

        let shutdown_token = CancellationToken::new();
        let span = Span::current();

        // If the caller wishes to receive forwarded output, spawn a task to forward output to the
        // sender (provided by the caller).
        if let Some(output_tx) = output_tx {
            tokio::spawn(forward_output_task(
                output_subscriber,
                output_tx,
                span,
                shutdown_token.clone(),
            ));
        }

        // Submit the request message
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );

        // Root reply mailbox will receive a reply if nobody is listening to the channel `subject`
        let mut root_subscriber = self.nats.subscribe(reply_mailbox_root.clone()).await?;

        // NOTE(nick,fletcher): based on the provided request mode, we will either communicate user core nats or
        // jetstream. We neither like nor endorse this behavior. This method should probably be broken up in the
        // future to cleanly separate core nats and jetstream use.
        match request_mode {
            RequestMode::Core => {
                self.nats
                    .publish_with_reply_and_headers(
                        subject,
                        reply_mailbox_root.clone(),
                        propagation::empty_injected_headers(),
                        msg.into(),
                    )
                    .await?
            }
            RequestMode::Jetstream => {
                let mut headers = propagation::empty_injected_headers();
                headers.insert(REPLY_INBOX_HEADER_NAME, reply_mailbox_root.clone());

                self.context
                    .publish_with_headers(subject, headers, msg.into())
                    .await
                    // If `Err` then message failed to publish
                    .map_err(|err| ClientError::Transport(Box::new(err)))?
                    .await
                    // If `Err` then NATS server failed to ack
                    .map_err(|err| ClientError::Transport(Box::new(err)))?;
            }
        }

        let span = Span::current();

        tokio::select! {
            // Wait for one message on the result reply mailbox
            result = result_subscriber.try_next() => {
                shutdown_token.cancel();

                root_subscriber.unsubscribe_after(0).await?;
                result_subscriber.unsubscribe_after(0).await?;
                match result? {
                    Some(result) => {
                        span.follows_from(result.process_span);
                        Ok(result.payload)
                    }
                    None => Err(ClientError::NoResult),
                }
            }
            maybe_msg = root_subscriber.next() => {
                shutdown_token.cancel();

                match &maybe_msg {
                    Some(msg) => {
                        propagation::associate_current_span_from_headers(msg.headers());
                        error!(
                            subject = reply_mailbox_root,
                            msg = ?msg,
                            "received an unexpected message or error on reply subject prefix"
                        )
                    }
                    None => {
                        error!(
                            subject = reply_mailbox_root,
                            "reply subject prefix subscriber unexpectedly closed"
                        )
                    }
                };

                // In all cases, we're considering a message on this subscriber to be fatal and
                // will return with an error
                Err(ClientError::PublishingFailed(maybe_msg.ok_or(ClientError::RootConnectionClosed)?))
            }
        }
    }
}

async fn forward_output_task(
    mut output_subscriber: Subscriber<OutputStream>,
    output_tx: mpsc::Sender<OutputStream>,
    request_span: Span,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            Some(msg) = output_subscriber.next() => {
                match msg {
                    Ok(output) => {
                        output.process_span.follows_from(&request_span);
                        if let Err(err) = output_tx.send(output.payload).await {
                            warn!(error = ?err, "output forwarder failed to send message on channel");
                        }
                    }
                    Err(err) => {
                        warn!(error = ?err, "output forwarder received an error on its subscriber")
                    }
                }
            }
            _ = shutdown_token.cancelled() => break,
            else => break,
        }
    }
    if let Err(err) = output_subscriber.unsubscribe_after(0).await {
        warn!(error = ?err, "error when unsubscribing from output subscriber");
    }
}
