use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_core::{
    nats_command_run_subject, nats_confirmation_subject, nats_resolver_function_subject,
    nats_subject, nats_workflow_resolve_subject, reply_mailbox_for_output,
    reply_mailbox_for_result,
};

mod subscription;

use subscription::{Subscription, SubscriptionError};

pub use cyclone_core::{
    CommandRunRequest, CommandRunResultSuccess, ComponentKind, ComponentView, ConfirmationRequest,
    ConfirmationResultSuccess, EncryptionKey, EncryptionKeyError, FunctionResult,
    FunctionResultFailure, OutputStream, ResolverFunctionComponent, ResolverFunctionRequest,
    ResolverFunctionResponseType, ResolverFunctionResultSuccess, ResourceStatus,
    SensitiveContainer, WorkflowResolveRequest, WorkflowResolveResultSuccess,
};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no function result from cyclone; bug!")]
    NoResult,
    #[error("result error")]
    Result(#[from] SubscriptionError),
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
    subject_prefix: Option<Arc<String>>,
}

impl Client {
    pub fn new(nats: NatsClient) -> Self {
        Self {
            nats,
            subject_prefix: None,
        }
    }

    pub fn with_subject_prefix(nats: NatsClient, subject_prefix: impl Into<String>) -> Self {
        Self {
            nats,
            subject_prefix: Some(Arc::new(subject_prefix.into())),
        }
    }

    #[instrument(name = "client.execute_confirmation", skip_all)]
    pub async fn execute_confirmation(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ConfirmationRequest,
    ) -> ClientResult<FunctionResult<ConfirmationResultSuccess>> {
        self.execute_request(
            nats_confirmation_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_confirmation_with_subject", skip_all)]
    pub async fn execute_confirmation_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ConfirmationRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<ConfirmationResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_resolver_function", skip_all)]
    pub async fn execute_resolver_function(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(
            nats_resolver_function_subject(self.subject_prefix()),
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
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_workflow_resolve", skip_all)]
    pub async fn execute_workflow_resolve(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &WorkflowResolveRequest,
    ) -> ClientResult<FunctionResult<WorkflowResolveResultSuccess>> {
        self.execute_request(
            nats_workflow_resolve_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_workflow_resolve_with_subject", skip_all)]
    pub async fn execute_workflow_resolve_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &WorkflowResolveRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<WorkflowResolveResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_command_run", skip_all)]
    pub async fn execute_command_run(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CommandRunRequest,
    ) -> ClientResult<FunctionResult<CommandRunResultSuccess>> {
        self.execute_request(
            nats_command_run_subject(self.subject_prefix()),
            output_tx,
            request,
        )
        .await
    }

    #[instrument(name = "client.execute_command_run_with_subject", skip_all)]
    pub async fn execute_command_run_with_subject(
        &self,
        output_tx: mpsc::Sender<OutputStream>,
        request: &CommandRunRequest,
        subject_suffix: impl AsRef<str>,
    ) -> ClientResult<FunctionResult<CommandRunResultSuccess>> {
        self.execute_request(
            nats_subject(self.subject_prefix(), subject_suffix),
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
            Subscription::new(self.nats.subscribe(result_subscription_subject).await?);

        // Construct a subscription stream for output messages
        let output_subscription_subject = reply_mailbox_for_output(&reply_mailbox_root);
        trace!(
            messaging.destination = &output_subscription_subject.as_str(),
            "subscribing for output messages"
        );
        let output_subscription =
            Subscription::new(self.nats.subscribe(output_subscription_subject).await?);
        // Spawn a task to forward output to the sender provided by the caller
        tokio::spawn(forward_output_task(output_subscription, output_tx));

        // Submit the request message
        let subject = subject.into();
        trace!(
            messaging.destination = &subject.as_str(),
            "publishing message"
        );
        self.nats
            .publish_with_reply_or_headers(subject, Some(reply_mailbox_root.as_str()), None, msg)
            .await?;

        // Wait for one message on the result reply mailbox
        let result = result_subscription
            .try_next()
            .await?
            .ok_or(ClientError::NoResult)?;
        result_subscription.unsubscribe().await?;

        Ok(result)
    }

    /// Gets a reference to the client's subject prefix.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.subject_prefix.as_deref().map(String::as_str)
    }
}

async fn forward_output_task(
    mut output_subscription: Subscription<OutputStream>,
    output_tx: mpsc::Sender<OutputStream>,
) {
    while let Some(msg) = output_subscription.next().await {
        match msg {
            Ok(output) => {
                if let Err(err) = output_tx.send(output).await {
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

    use si_data_nats::NatsConfig;
    use test_log::test;
    use tokio::task::JoinHandle;
    use uuid::Uuid;
    use veritech_server::{
        Config, CycloneSpec, Instance, LocalUdsInstance, Server, ServerError, StandardConfig,
    };

    use super::*;

    fn nats_config() -> NatsConfig {
        let mut config = NatsConfig::default();
        if let Ok(value) = env::var("SI_TEST_NATS_URL") {
            config.url = value;
        }
        config
    }

    async fn nats() -> NatsClient {
        NatsClient::new(&nats_config())
            .await
            .expect("failed to connect to NATS")
    }

    fn nats_prefix() -> String {
        Uuid::new_v4().as_simple().to_string()
    }

    async fn veritech_server_for_uds_cyclone(subject_prefix: String) -> Server {
        let cyclone_spec = CycloneSpec::LocalUds(
            LocalUdsInstance::spec()
                .try_cyclone_cmd_path("../../target/debug/cyclone")
                .expect("failed to setup cyclone_cmd_path")
                .cyclone_decryption_key_path("../../lib/cyclone-server/src/dev.decryption.key")
                .try_lang_server_cmd_path("../../bin/lang-js/target/lang-js")
                .expect("failed to setup lang_js_cmd_path")
                .all_endpoints()
                .build()
                .expect("failed to build cyclone spec"),
        );
        let config = Config::builder()
            .nats(nats_config())
            .subject_prefix(subject_prefix)
            .cyclone_spec(cyclone_spec)
            .build()
            .expect("failed to build spec");
        Server::for_cyclone_uds(config)
            .await
            .expect("failed to create server")
    }

    async fn client(subject_prefix: String) -> Client {
        Client::with_subject_prefix(nats().await, subject_prefix)
    }

    async fn run_veritech_server_for_uds_cyclone(
        subject_prefix: String,
    ) -> JoinHandle<Result<(), ServerError>> {
        tokio::spawn(veritech_server_for_uds_cyclone(subject_prefix).await.run())
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
            code_base64: base64::encode(
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
                panic!("function did not succeed and should have: {:?}", failure)
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
            ResolverFunctionResponseType::PropObject,
        ] {
            let value = match response_type {
                ResolverFunctionResponseType::Array => serde_json::json!({ "value": [1, 2, 3, 4] }),
                ResolverFunctionResponseType::Integer => serde_json::json!({ "value": 31337 }),
                ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": true }),
                ResolverFunctionResponseType::String => {
                    serde_json::json!({ "value": "a string is a sequence of characters" })
                }
                ResolverFunctionResponseType::Map | ResolverFunctionResponseType::PropObject => {
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
                code_base64: base64::encode(
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
            ResolverFunctionResponseType::PropObject,
        ] {
            let value = match response_type {
                ResolverFunctionResponseType::Array => serde_json::json!({ "value": "foo"}),
                ResolverFunctionResponseType::Integer => serde_json::json!({ "value": "a string" }),
                ResolverFunctionResponseType::Boolean => serde_json::json!({ "value": "a string" }),
                ResolverFunctionResponseType::String => serde_json::json!({ "value": 12345 }),
                ResolverFunctionResponseType::Map | ResolverFunctionResponseType::PropObject => {
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
                code_base64: base64::encode(
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
    async fn executes_simple_confirmation() {
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

        let request = ConfirmationRequest {
            execution_id: "7868".to_string(),
            handler: "confirmItOut".to_string(),
            component: ComponentView {
                properties: serde_json::json!({"pkg": "cider"}),
                kind: ComponentKind::Standard,
            },
            code_base64: base64::encode("function confirmItOut(component) { return { success: false, recommendedActions: ['vai te catar'] } }")
        };

        let result = client
            .execute_confirmation(tx, &request)
            .await
            .expect("failed to execute confirmation");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "7868");
                assert!(!success.success);
                assert_eq!(success.recommended_actions, vec!["vai te catar".to_owned()]);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_workflow_resolve() {
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

        let request = WorkflowResolveRequest {
            execution_id: "112233".to_string(),
            handler: "workItOut".to_string(),
            // TODO(fnichol): rewrite this function once we settle on contract
            code_base64: base64::encode("function workItOut() { return { name: 'mc fioti', kind: 'vacina butantan - https://www.youtube.com/watch?v=yQ8xJHuW7TY', steps: [] }; }"),
            args: Default::default(),
        };

        let result = client
            .execute_workflow_resolve(tx, &request)
            .await
            .expect("failed to execute workflow resolve");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "112233");
                // TODO(fnichol): add more assertions as we add fields
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }
}
