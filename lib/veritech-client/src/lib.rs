use futures::{StreamExt, TryStreamExt};
use nats_subscriber::{Subscriber, SubscriberError};
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use veritech_core::{
    nats_action_run_subject, nats_reconciliation_subject, nats_resolver_function_subject,
    nats_schema_variant_definition_subject, nats_subject, nats_validation_subject,
    reply_mailbox_for_keep_alive, reply_mailbox_for_output, reply_mailbox_for_result,
    FINAL_MESSAGE_HEADER_KEY,
};

pub use cyclone_core::{
    encrypt_value_tree, ActionRunRequest, ActionRunResultSuccess, BeforeFunction, ComponentKind,
    ComponentView, CycloneValueDecryptError, CycloneValueEncryptError, FunctionResult,
    FunctionResultFailure, OutputStream, ReconciliationRequest, ReconciliationResultSuccess,
    ResolverFunctionComponent, ResolverFunctionRequest, ResolverFunctionResponseType,
    ResolverFunctionResultSuccess, ResourceStatus, SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess, SensitiveContainer, ValidationRequest,
    ValidationResultSuccess,
};
pub use si_crypto::{CycloneEncryptionKey, CycloneEncryptionKeyError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no keep alive from cyclone")]
    NoKeepAlive,
    #[error("no function result from cyclone; bug!")]
    NoResult,
    #[error("unable to publish message: {0:?}")]
    PublishingFailed(si_data_nats::Message),
    #[error("root connection closed")]
    RootConnectionClosed,
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
}

impl Client {
    pub fn new(nats: NatsClient) -> Self {
        Self { nats }
    }

    fn nats_subject_prefix(&self) -> Option<&str> {
        self.nats.metadata().subject_prefix()
    }

    #[instrument(name = "client.execute_resolver_function", level = "info", skip_all)]
    pub async fn execute_resolver_function(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            nats_resolver_function_subject(self.nats_subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(
        name = "client.execute_resolver_function_with_subject",
        level = "info",
        skip_all
    )]
    pub async fn execute_resolver_function_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            nats_subject(self.nats_subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_validation", level = "info", skip_all)]
    pub async fn execute_validation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ValidationRequest,
    ) -> ClientResult<FunctionResult<ValidationResultSuccess>> {
        self.execute_request(
            nats_validation_subject(self.nats_subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(
        name = "client.execute_validation_with_subject",
        level = "info",
        skip_all
    )]
    pub async fn execute_validation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ValidationResultSuccess,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ValidationResultSuccess>> {
        self.execute_request(
            nats_subject(self.nats_subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_action_run", level = "info", skip_all)]
    pub async fn execute_action_run(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ActionRunRequest,
    ) -> ClientResult<FunctionResult<ActionRunResultSuccess>> {
        self.execute_request(
            nats_action_run_subject(self.nats_subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(
        name = "client.execute_action_run_with_subject",
        level = "info",
        skip_all
    )]
    pub async fn execute_action_run_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ActionRunRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ActionRunResultSuccess>> {
        self.execute_request(
            nats_subject(self.nats_subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_reconciliation", level = "info" skip_all)]
    pub async fn execute_reconciliation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ReconciliationRequest,
    ) -> ClientResult<FunctionResult<ReconciliationResultSuccess>> {
        self.execute_request(
            nats_reconciliation_subject(self.nats_subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(
        name = "client.execute_reconciliation_with_subject",
        level = "info",
        skip_all
    )]
    pub async fn execute_reconciliation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ReconciliationRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ReconciliationResultSuccess>> {
        self.execute_request(
            nats_subject(self.nats_subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_reconciliation", level = "info", skip_all)]
    pub async fn execute_schema_variant_definition(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &SchemaVariantDefinitionRequest,
    ) -> ClientResult<FunctionResult<SchemaVariantDefinitionResultSuccess>> {
        self.execute_request(
            nats_schema_variant_definition_subject(self.nats_subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(
        name = "client.execute_reconciliation_with_subject",
        level = "info",
        skip_all
    )]
    pub async fn execute_schema_variant_definition_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &SchemaVariantDefinitionRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<SchemaVariantDefinitionResultSuccess>> {
        self.execute_request(
            nats_subject(self.nats_subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    async fn execute_request<R, S>(
        &self,
        subject: impl Into<String>,
        output_tx: mpsc::Sender<OutputStream>,
        request: &R,
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

        // Construct a subscriber stream for keep-alive messages
        let keep_alive_subscriber_subject = reply_mailbox_for_keep_alive(&reply_mailbox_root);
        trace!(
            messaging.destination = &keep_alive_subscriber_subject.as_str(),
            "subscribing for keep-alive messages"
        );
        let mut keep_alive_subscriber: Subscriber<()> =
            Subscriber::create(keep_alive_subscriber_subject)
                .final_message_header_key(FINAL_MESSAGE_HEADER_KEY)
                .start(&self.nats)
                .await?;

        let shutdown_token = CancellationToken::new();
        let span = Span::current();
        // Spawn a task to forward output to the sender provided by the caller
        tokio::spawn(forward_output_task(
            output_subscriber,
            output_tx,
            span,
            shutdown_token.clone(),
        ));

        // Submit the request message
        let subject = subject.into();
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );

        // Root reply mailbox will receive a reply if nobody is listening to the channel `subject`
        let mut root_subscriber = self.nats.subscribe(reply_mailbox_root.clone()).await?;

        self.nats
            .publish_with_reply_and_headers(
                subject,
                reply_mailbox_root.clone(),
                propagation::empty_injected_headers(),
                msg.into(),
            )
            .await?;

        let span = Span::current();

        loop {
            tokio::select! {
                _ = keep_alive_subscriber.next() => {
                    info!("Heartbeat from veritech");
                    continue;
                }
                // Abort if no keep-alive for too long
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    return Err(ClientError::NoKeepAlive);
                }
                // Wait for one message on the result reply mailbox
                result = result_subscriber.try_next() => {
                    shutdown_token.cancel();

                    root_subscriber.unsubscribe_after(0).await?;
                    result_subscriber.unsubscribe_after(0).await?;
                    match result? {
                        Some(result) => {
                            span.follows_from(result.process_span);
                            return Ok(result.payload);
                        }
                        None => return Err(ClientError::NoResult),
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
                    return Err(ClientError::PublishingFailed(maybe_msg.ok_or(ClientError::RootConnectionClosed)?));
                }
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
