use futures::{StreamExt, TryStreamExt};
use nats_subscriber::{SubscriberError, Subscription};
use serde::{de::DeserializeOwned, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

use veritech_core::{
    nats_action_run_subject, nats_reconciliation_subject, nats_resolver_function_subject,
    nats_schema_variant_definition_subject, nats_subject, nats_validation_subject,
    reply_mailbox_for_output, reply_mailbox_for_result, FINAL_MESSAGE_HEADER_KEY,
};

pub use cyclone_core::{
    ActionRunRequest, ActionRunResultSuccess, ComponentKind, ComponentView, EncryptionKey,
    EncryptionKeyError, FunctionResult, FunctionResultFailure, OutputStream, ReconciliationRequest,
    ReconciliationResultSuccess, ResolverFunctionComponent, ResolverFunctionRequest,
    ResolverFunctionResponseType, ResolverFunctionResultSuccess, ResourceStatus,
    SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess, SensitiveContainer,
    ValidationRequest, ValidationResultSuccess,
};
use si_data_nats::NatsClient;

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

    #[instrument(name = "client.execute_resolver_function", skip_all)]
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

    #[instrument(name = "client.execute_resolver_function_with_subject", skip_all)]
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

    #[instrument(name = "client.execute_validation", skip_all)]
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

    #[instrument(name = "client.execute_validation_with_subject", skip_all)]
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

    #[instrument(name = "client.execute_action_run", skip_all)]
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

    #[instrument(name = "client.execute_action_run_with_subject", skip_all)]
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

    #[instrument(name = "client.execute_reconciliation", skip_all)]
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

    #[instrument(name = "client.execute_reconciliation_with_subject", skip_all)]
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

    #[instrument(name = "client.execute_reconciliation", skip_all)]
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

    #[instrument(name = "client.execute_reconciliation_with_subject", skip_all)]
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

        // Construct a subscription stream for the result
        let result_subscription_subject = reply_mailbox_for_result(&reply_mailbox_root);
        trace!(
            messaging.destination = &result_subscription_subject.as_str(),
            "subscribing for result messages"
        );
        let mut result_subscription: Subscription<FunctionResult<S>> =
            Subscription::create(result_subscription_subject)
                .final_message_header_key(FINAL_MESSAGE_HEADER_KEY)
                .start(&self.nats)
                .await?;

        // Construct a subscription stream for output messages
        let output_subscription_subject = reply_mailbox_for_output(&reply_mailbox_root);
        trace!(
            messaging.destination = &output_subscription_subject.as_str(),
            "subscribing for output messages"
        );
        let output_subscription = Subscription::create(output_subscription_subject)
            .final_message_header_key(FINAL_MESSAGE_HEADER_KEY)
            .start(&self.nats)
            .await?;

        // Spawn a task to forward output to the sender provided by the caller
        tokio::spawn(forward_output_task(output_subscription, output_tx));

        // Submit the request message
        let subject = subject.into();
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );

        // Root reply mailbox will receive a reply if nobody is listening to the channel `subject`
        let mut root_subscription = self.nats.subscribe(reply_mailbox_root.clone()).await?;

        self.nats
            .publish_with_reply(subject, reply_mailbox_root.clone(), msg)
            .await?;

        tokio::select! {
            // Wait for one message on the result reply mailbox
            result = result_subscription.try_next() => {
                root_subscription.unsubscribe_after(0).await?;
                result_subscription.unsubscribe_after(0).await?;
                match result? {
                    Some(result) => Ok(result.payload),
                    None => Err(ClientError::NoResult)
                }
            }
            reply = root_subscription.next() => {
                match &reply {
                    Some(maybe_msg) => {
                        error!(
                            subject = reply_mailbox_root,
                            maybe_msg = ?maybe_msg,
                            "received an unexpected message or error on reply subject prefix"
                        )
                    }
                    None => {
                        error!(
                            subject = reply_mailbox_root,
                            "reply subject prefix subscription unexpectedly closed"
                        )
                    }
                };

                // In all cases, we're considering a message on this subscription to be fatal and
                // will return with an error
                Err(ClientError::PublishingFailed(reply.ok_or(ClientError::RootConnectionClosed)?))
            }
        }
    }
}

async fn forward_output_task(
    mut output_subscription: Subscription<OutputStream>,
    output_tx: mpsc::Sender<OutputStream>,
) {
    while let Some(msg) = output_subscription.next().await {
        match msg {
            Ok(output) => {
                if let Err(err) = output_tx.send(output.payload).await {
                    warn!(error = ?err, "output forwarder failed to send message on channel");
                }
            }
            Err(err) => {
                warn!(error = ?err, "output forwarder received an error on its subscription")
            }
        }
    }
    if let Err(err) = output_subscription.unsubscribe_after(0).await {
        warn!(error = ?err, "error when unsubscribing from output subscription");
    }
}
