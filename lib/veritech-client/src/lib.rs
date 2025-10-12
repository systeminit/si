use cyclone_core::CycloneRequestable;
pub use cyclone_core::{
    ActionRunRequest,
    ActionRunResultSuccess,
    BeforeFunction,
    ComponentKind,
    ComponentView,
    ComponentViewWithGeometry,
    FunctionResult,
    FunctionResultFailure,
    FunctionResultFailureErrorKind,
    KillExecutionRequest,
    ManagementFuncStatus,
    ManagementRequest,
    ManagementResultSuccess,
    OutputStream,
    ResolverFunctionComponent,
    ResolverFunctionRequest,
    ResolverFunctionResponseType,
    ResolverFunctionResultSuccess,
    ResourceStatus,
    SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess,
    SensitiveContainer,
    ValidationRequest,
    ValidationResultSuccess,
};
use futures::{
    StreamExt,
    TryStreamExt,
};
use nats_std::header;
use nats_subscriber::{
    Subscriber,
    SubscriberError,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_data_nats::{
    NatsClient,
    Subject,
    jetstream,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use veritech_core::{
    FINAL_MESSAGE_HEADER_KEY,
    GetNatsSubjectFor,
    reply_mailbox_for_output,
    reply_mailbox_for_result,
};
pub use veritech_core::{
    VeritechValueEncryptError,
    encrypt_value_tree,
};

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
    #[error("tokio join error: {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("transport error: {0}")]
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
        self.execute_jetstream_request(output_tx, request, workspace_id, change_set_id)
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
        self.execute_jetstream_request(output_tx, request, workspace_id, change_set_id)
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
        self.execute_jetstream_request(output_tx, request, workspace_id, change_set_id)
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
        self.execute_jetstream_request(output_tx, request, workspace_id, change_set_id)
            .await
    }

    #[instrument(
        name = "veritech_client.execute_management",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = change_set_id,
            si.workspace.id = workspace_id,
        ),
    )]
    pub async fn execute_management(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ManagementRequest,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<ManagementResultSuccess>> {
        self.execute_jetstream_request(output_tx, request, workspace_id, change_set_id)
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
            request.nats_subject(self.nats_subject_prefix(), None, None),
            None,
            request,
            RequestMode::Core,
        )
        .await
    }

    async fn execute_jetstream_request<R>(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &R,
        workspace_id: &str,
        change_set_id: &str,
    ) -> ClientResult<FunctionResult<R::Response>>
    where
        R: Serialize + CycloneRequestable + GetNatsSubjectFor,
        R::Response: DeserializeOwned,
    {
        self.execute_request(
            request.nats_subject(
                self.nats_subject_prefix(),
                Some(workspace_id),
                Some(change_set_id),
            ),
            Some(output_tx),
            request,
            RequestMode::Jetstream,
        )
        .await
    }

    async fn execute_request<R: Serialize + CycloneRequestable<Response: DeserializeOwned>>(
        &self,
        subject: Subject,
        output_tx: Option<mpsc::Sender<OutputStream>>,
        request: &R,
        request_mode: RequestMode,
    ) -> ClientResult<FunctionResult<R::Response>> {
        // Subscribe to responses and send the request. These unsubscribe when dropped.
        let (root_subscriber, mut result_subscriber, output_subscriber) =
            self.send_request(subject, request, request_mode).await?;

        // Forward output messages in the background
        let cancel_output = CancellationToken::new();
        if let Some(output_tx) = output_tx {
            let cancel_output = cancel_output.clone();
            tokio::spawn(cancel_output.run_until_cancelled_owned(forward_output(
                output_subscriber,
                output_tx,
                Span::current(),
            )));
        }

        tokio::select! {
            result = result_subscriber.try_next() => {
                let result = result?.ok_or(ClientError::NoResult)?;
                Span::current().follows_from(result.process_span);
                Ok(result.payload)
            }

            // Because the channel never responds on success, we have to await this simultaneously
            // with the result. If it *does* fail, we cancel the output forwarder as well.
            err = check_for_send_error(root_subscriber) => {
                cancel_output.cancel();
                Err(err)
            }
        }
    }

    async fn send_request<R: Serialize + CycloneRequestable<Response: DeserializeOwned>>(
        &self,
        subject: Subject,
        request: &R,
        request_mode: RequestMode,
    ) -> ClientResult<(
        si_data_nats::Subscriber,
        Subscriber<FunctionResult<R::Response>>,
        Subscriber<OutputStream>,
    )> {
        let msg = serde_json::to_vec(request).map_err(ClientError::JSONSerialize)?;
        let reply_mailbox_root = self.nats.new_inbox();

        // Subscribe first! So we're already listening when the messages come in.

        // Subscribe to the result
        let result_subscriber_subject = reply_mailbox_for_result(&reply_mailbox_root);
        trace!(
            messaging.destination = &result_subscriber_subject.as_str(),
            "subscribing for result messages"
        );
        let result_subscriber = Subscriber::create(result_subscriber_subject)
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

        // Root reply mailbox will receive a reply if nobody is listening to the channel `subject`
        let root_subscriber = self.nats.subscribe(reply_mailbox_root.clone()).await?;

        // Submit the request message
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );

        // NOTE(nick,fletcher): based on the provided request mode, we will either communicate user core nats or
        // jetstream. We neither like nor endorse this behavior. This method should probably be broken up in the
        // future to cleanly separate core nats and jetstream use.
        match request_mode {
            RequestMode::Core => {
                self.nats
                    .publish_with_reply_and_headers(
                        subject,
                        reply_mailbox_root,
                        propagation::empty_injected_headers(),
                        msg.into(),
                    )
                    .await?
            }
            RequestMode::Jetstream => {
                let mut headers = propagation::empty_injected_headers();
                header::insert_reply_inbox(&mut headers, &reply_mailbox_root);

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

        Ok((root_subscriber, result_subscriber, output_subscriber))
    }
}

async fn check_for_send_error(mut root_subscriber: si_data_nats::Subscriber) -> ClientError {
    // Return an error if no one is listening.
    match root_subscriber.next().await {
        Some(msg) => {
            propagation::associate_current_span_from_headers(msg.headers());
            error!(
                subject = root_subscriber.subject(),
                msg = ?msg,
                "received an unexpected message or error on reply subject prefix"
            );
            ClientError::PublishingFailed(msg)
        }
        None => {
            error!(
                subject = root_subscriber.subject(),
                "reply subject prefix subscriber unexpectedly closed"
            );
            ClientError::RootConnectionClosed
        }
    }
}

async fn forward_output(
    mut output_subscriber: Subscriber<OutputStream>,
    output_tx: mpsc::Sender<OutputStream>,
    request_span: Span,
) {
    // Listen for messages
    while let Some(msg) = tokio::select! {
        next = output_subscriber.next() => next,
        // Exit early if the output channel is closed; no one cares about the output!
        _ = output_tx.closed() => {
            warn!("cancelling output forwarder: receiver closed early");
            None
        }
    } {
        match msg {
            Ok(output) => {
                output.process_span.follows_from(&request_span);
                if let Err(err) = output_tx.send(output.payload).await {
                    warn!(si.error.message = ?err, "output forwarder failed to send message on channel");
                }
            }
            Err(err) => {
                warn!(si.error.message = ?err, "output forwarder received an error on its subscriber");
            }
        }
    }
}
