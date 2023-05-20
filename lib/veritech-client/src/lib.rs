use futures::{StreamExt, TryStreamExt};
use nats_subscriber::{SubscriberError, Subscription};
use serde::{de::DeserializeOwned, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

use veritech_core::{
    nats_action_run_subject, nats_reconciliation_subject, nats_resolver_function_subject,
    nats_subject, nats_validation_subject, reply_mailbox_for_output, reply_mailbox_for_result,
    FINAL_MESSAGE_HEADER_KEY,
};

pub use cyclone_core::{
    ActionRunRequest, ActionRunResultSuccess, ComponentKind, ComponentView, EncryptionKey,
    EncryptionKeyError, FunctionResult, FunctionResultFailure, OutputStream, ReconciliationRequest,
    ReconciliationResultSuccess, ResolverFunctionComponent, ResolverFunctionRequest,
    ResolverFunctionResponseType, ResolverFunctionResultSuccess, ResourceStatus,
    SensitiveContainer, ValidationRequest, ValidationResultSuccess,
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
            .publish_with_reply_or_headers(subject, Some(reply_mailbox_root.clone()), None, msg)
            .await?;

        tokio::select! {
            // Wait for one message on the result reply mailbox
            result = result_subscription.try_next() => {
                root_subscription.unsubscribe().await?;
                result_subscription.unsubscribe().await?;
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
                Err(ClientError::PublishingFailed(reply.ok_or(ClientError::RootConnectionClosed)??))
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
    if let Err(err) = output_subscription.unsubscribe().await {
        warn!(error = ?err, "error when unsubscribing from output subscription");
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use std::env;

    use base64::{engine::general_purpose, Engine};
    use si_data_nats::NatsConfig;
    use test_log::test;
    use tokio::task::JoinHandle;
    use uuid::Uuid;
    use veritech_server::{
        Config, CycloneSpec, Instance, LocalUdsInstance, Server, ServerError, StandardConfig,
    };

    use super::*;

    fn nats_config(subject_prefix: String) -> NatsConfig {
        let mut config = NatsConfig::default();
        if let Ok(value) = env::var("SI_TEST_NATS_URL") {
            config.url = value;
        }
        config.subject_prefix = Some(subject_prefix);
        config
    }

    async fn nats(subject_prefix: String) -> NatsClient {
        NatsClient::new(&nats_config(subject_prefix))
            .await
            .expect("failed to connect to NATS")
    }

    fn nats_prefix() -> String {
        Uuid::new_v4().as_simple().to_string()
    }

    async fn veritech_server_for_uds_cyclone(subject_prefix: String) -> Server {
        let mut config_file = veritech_server::ConfigFile::default_local_uds();
        veritech_server::detect_and_configure_development(&mut config_file)
            .expect("failed to determine test configuration");

        let cyclone_spec = CycloneSpec::LocalUds(
            LocalUdsInstance::spec()
                .try_cyclone_cmd_path(config_file.cyclone.cyclone_cmd_path())
                .expect("failed to setup cyclone_cmd_path")
                .cyclone_decryption_key_path(config_file.cyclone.cyclone_decryption_key_path())
                .try_lang_server_cmd_path(config_file.cyclone.lang_server_cmd_path())
                .expect("failed to setup lang_js_cmd_path")
                .all_endpoints()
                .build()
                .expect("failed to build cyclone spec"),
        );
        let config = Config::builder()
            .nats(nats_config(subject_prefix.clone()))
            .cyclone_spec(cyclone_spec)
            .build()
            .expect("failed to build spec");
        Server::for_cyclone_uds(config)
            .await
            .expect("failed to create server")
    }

    async fn client(subject_prefix: String) -> Client {
        Client::new(nats(subject_prefix).await)
    }

    async fn run_veritech_server_for_uds_cyclone(
        subject_prefix: String,
    ) -> JoinHandle<Result<(), ServerError>> {
        tokio::spawn(veritech_server_for_uds_cyclone(subject_prefix).await.run())
    }

    fn base64_encode(input: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD_NO_PAD.encode(input)
    }

    #[test(tokio::test)]
    async fn executes_simple_resolver_function() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "numberOfInputs".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: serde_json::json!({ "foo": "bar", "baz": "quux" }),
                    kind: ComponentKind::Standard,
                },
                parents: vec![],
            },
            response_type: ResolverFunctionResponseType::Integer,
            code_base64: base64_encode(
                "function numberOfInputs(input) { return Object.keys(input)?.length ?? 0; }",
            ),
        };

        let result = client
            .execute_resolver_function(tx, &request)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "1234");
                assert_eq!(success.data, serde_json::json!(2));
                assert!(!success.unset);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {failure:?}")
            }
        }
    }

    #[test(tokio::test)]
    async fn type_checks_resolve_function() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        for response_type in [
            ResolverFunctionResponseType::Array,
            ResolverFunctionResponseType::Integer,
            ResolverFunctionResponseType::Boolean,
            ResolverFunctionResponseType::String,
            ResolverFunctionResponseType::Map,
            ResolverFunctionResponseType::Object,
        ] {
            let value = match response_type {
                ResolverFunctionResponseType::Array => serde_json::json!({ "value": [1, 2, 3, 4] }),
                ResolverFunctionResponseType::Integer => serde_json::json!({ "value": 31337 }),
                ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": true }),
                ResolverFunctionResponseType::String => {
                    serde_json::json!({ "value": "a string is a sequence of characters" })
                }
                ResolverFunctionResponseType::Map | ResolverFunctionResponseType::Object => {
                    serde_json::json!({ "value": { "an_object": "has keys" } })
                }
                _ => serde_json::json!({ "value": null }),
            };
            // Not going to check output here--we aren't emitting anything
            let (tx, mut rx) = mpsc::channel(64);
            tokio::spawn(async move {
                while let Some(output) = rx.recv().await {
                    info!("output: {:?}", output)
                }
            });

            let request = ResolverFunctionRequest {
                execution_id: "1234".to_string(),
                handler: "returnInputValue".to_string(),
                component: ResolverFunctionComponent {
                    data: ComponentView {
                        properties: value.clone(),
                        kind: ComponentKind::Standard,
                    },
                    parents: vec![],
                },
                response_type,
                code_base64: base64_encode(
                    "function returnInputValue(input) { return input.value; }",
                ),
            };

            let result = client
                .execute_resolver_function(tx, &request)
                .await
                .expect("failed to execute resolver function");

            match result {
                FunctionResult::Success(success) => {
                    assert_eq!(success.execution_id, "1234");
                    if let serde_json::Value::Object(inner) = value {
                        let value = inner.get("value").expect("value should exist").clone();
                        assert_eq!(value, success.data);
                    } else {
                        panic!("no value in return data :(")
                    }
                }
                FunctionResult::Failure(_) => {
                    panic!("should have failed :(");
                }
            }
        }

        for response_type in [
            ResolverFunctionResponseType::Array,
            ResolverFunctionResponseType::Integer,
            ResolverFunctionResponseType::Boolean,
            ResolverFunctionResponseType::String,
            ResolverFunctionResponseType::Map,
            ResolverFunctionResponseType::Object,
        ] {
            let value = match response_type {
                ResolverFunctionResponseType::Array => serde_json::json!({ "value": "foo"}),
                ResolverFunctionResponseType::Integer => serde_json::json!({ "value": "a string" }),
                ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": "a string" }),
                ResolverFunctionResponseType::String => serde_json::json!({ "value": 12345 }),
                ResolverFunctionResponseType::Map | ResolverFunctionResponseType::Object => {
                    serde_json::json!({ "value": ["an_object", "has keys" ] })
                }
                _ => serde_json::json!({ "value": null }),
            };
            // Not going to check output here--we aren't emitting anything
            let (tx, mut rx) = mpsc::channel(64);
            tokio::spawn(async move {
                while let Some(output) = rx.recv().await {
                    info!("output: {:?}", output)
                }
            });

            let request = ResolverFunctionRequest {
                execution_id: "1234".to_string(),
                handler: "returnInputValue".to_string(),
                component: ResolverFunctionComponent {
                    data: ComponentView {
                        properties: value,
                        kind: ComponentKind::Standard,
                    },
                    parents: vec![],
                },
                response_type: response_type.clone(),
                code_base64: base64_encode(
                    "function returnInputValue(input) { return input.value; }",
                ),
            };

            let result = client
                .execute_resolver_function(tx, &request)
                .await
                .expect("failed to execute resolver function");

            match result {
                FunctionResult::Success(success) => {
                    dbg!(success, response_type);
                    panic!("should have failed :(");
                }
                FunctionResult::Failure(failure) => {
                    assert_eq!(failure.error.kind, "InvalidReturnType");
                    assert_eq!(failure.execution_id, "1234");
                }
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_validation() {
        let prefix = nats_prefix();
        run_veritech_server_for_uds_cyclone(prefix.clone()).await;
        let client = client(prefix).await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let request = ValidationRequest {
            execution_id: "31337".to_string(),
            handler: "isThirtyThree".to_string(),
            value: 33.into(),
            code_base64: base64_encode(
                "function isThirtyThree(value) { return { valid: value === 33 }; };",
            ),
        };

        let result = client
            .execute_validation(tx, &request)
            .await
            .expect("failed to execute validation");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "31337");
                assert!(success.valid);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {failure:?}")
            }
        }
    }
}
