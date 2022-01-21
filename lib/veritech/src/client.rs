use cyclone::{
    FunctionResult, OutputStream, QualificationCheckRequest, QualificationCheckResultSuccess,
    ResolverFunctionRequest, ResolverFunctionResultSuccess,
};
use futures::{StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use si_data::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

use self::subscription::{Subscription, SubscriptionError};
use crate::{reply_mailbox_for_output, reply_mailbox_for_result};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("nats error")]
    Nats(#[from] si_data::NatsError),
    #[error("no function result from cyclone; bug!")]
    NoResult,
    #[error("result error")]
    Result(#[from] SubscriptionError),
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

    #[instrument(name = "client.execute_resolver_function", skip_all)]
    pub async fn execute_resolver_function(
        &self,
        subject: impl Into<String>,
        output_tx: mpsc::Sender<OutputStream>,
        request: &ResolverFunctionRequest,
    ) -> ClientResult<FunctionResult<ResolverFunctionResultSuccess>> {
        self.execute_request(subject, output_tx, request).await
    }

    #[instrument(name = "client.execute_qualification_check", skip_all)]
    pub async fn execute_qualification_check(
        &self,
        subject: impl Into<String>,
        output_tx: mpsc::Sender<OutputStream>,
        request: &QualificationCheckRequest,
    ) -> ClientResult<FunctionResult<QualificationCheckResultSuccess>> {
        self.execute_request(subject, output_tx, request).await
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
        let mut result_subscription: Subscription<FunctionResult<S>> = Subscription::new(
            self.nats
                .subscribe(reply_mailbox_for_result(&reply_mailbox_root))
                .await?,
        );

        // Construct a subscription stream for output messages
        let output_subscription = Subscription::new(
            self.nats
                .subscribe(reply_mailbox_for_output(&reply_mailbox_root))
                .await?,
        );
        // Spawn a task to forward output to the sender provided by the caller
        tokio::spawn(forward_output_task(output_subscription, output_tx));

        // Submit the request message
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

mod subscription {
    use std::{
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures::{Stream, StreamExt};
    use futures_lite::FutureExt;
    use pin_project_lite::pin_project;
    use serde::de::DeserializeOwned;
    use si_data::nats;
    use telemetry::prelude::*;
    use thiserror::Error;

    use crate::FINAL_MESSAGE_HEADER_KEY;

    #[derive(Error, Debug)]
    pub enum SubscriptionError {
        #[error("failed to deserialize json message")]
        JSONDeserialize(#[source] serde_json::Error),
        #[error("nats io error when reading from subscription")]
        NatsIo(#[source] si_data::NatsError),
        #[error("failed to unsubscribe from nats subscription")]
        NatsUnsubscribe(#[source] si_data::NatsError),
        #[error("the nats subscription closed before seeing a final message")]
        UnexpectedNatsSubscriptionClosed,
    }

    pin_project! {
        #[derive(Debug)]
        pub struct Subscription<T> {
            #[pin]
            inner: nats::Subscription,
            _phantom: PhantomData<T>,
        }
    }

    impl<T> Subscription<T> {
        pub fn new(inner: nats::Subscription) -> Self {
            Subscription {
                inner,
                _phantom: PhantomData,
            }
        }

        pub async fn unsubscribe(self) -> Result<(), SubscriptionError> {
            self.inner
                .unsubscribe()
                .await
                .map_err(SubscriptionError::NatsUnsubscribe)
        }
    }

    impl<T> Stream for Subscription<T>
    where
        T: DeserializeOwned,
    {
        type Item = Result<T, SubscriptionError>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let mut this = self.project();

            match this.inner.next().poll(cx) {
                // Convert this NATS message into the cyclone request type `T` and return any
                // errors for the caller to decide how to proceed (i.e. does the caller fail on
                // first error, ignore error items, etc.)
                Poll::Ready(Some(Ok(nats_msg))) => {
                    // If the NATS message has a final message header, then treat this as an
                    // end-of-stream marker and close our stream.
                    if let Some(headers) = nats_msg.headers() {
                        if headers.keys().any(|key| key == FINAL_MESSAGE_HEADER_KEY) {
                            trace!(
                                "{} header detected in NATS message, closing stream",
                                FINAL_MESSAGE_HEADER_KEY
                            );
                            return Poll::Ready(None);
                        }
                    }

                    let data = nats_msg.into_data();
                    match serde_json::from_slice::<T>(&data) {
                        // Deserializing from JSON into the target type was successful
                        Ok(msg) => Poll::Ready(Some(Ok(msg))),
                        // Deserializing failed
                        Err(err) => Poll::Ready(Some(Err(SubscriptionError::JSONDeserialize(err)))),
                    }
                }
                // A NATS error occurred (async error or other i/o)
                Poll::Ready(Some(Err(err))) => {
                    Poll::Ready(Some(Err(SubscriptionError::NatsIo(err))))
                }
                // We see no more messages on the subject, but we haven't seen a "final message"
                // yet, so this is an unexpected problem
                Poll::Ready(None) => Poll::Ready(Some(Err(
                    SubscriptionError::UnexpectedNatsSubscriptionClosed,
                ))),
                // Not ready, so...not ready!
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env};

    use cyclone::QualificationCheckComponent;
    use deadpool_cyclone::{instance::cyclone::LocalUdsInstance, Instance};
    use indoc::indoc;
    use si_data::NatsConfig;
    use si_settings::StandardConfig;
    use test_env_log::test;
    use tokio::task::JoinHandle;

    use super::*;
    use crate::{Config, CycloneSpec, Server, ServerError};

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

    async fn server_for_uds_cyclone() -> Server {
        let cyclone_spec = CycloneSpec::LocalUds(
            LocalUdsInstance::spec()
                .try_cyclone_cmd_path("../../target/debug/cyclone")
                .expect("failed to setup cyclone_cmd_path")
                .try_lang_server_cmd_path("../../bin/lang-js/target/lang-js")
                .expect("failed to setup lang_js_cmd_path")
                .resolver()
                .build()
                .expect("failed to build cyclone spec"),
        );
        let config = Config::builder()
            .nats(nats_config())
            .cyclone_spec(cyclone_spec)
            .build()
            .expect("failed to build spec");
        Server::for_cyclone_uds(config)
            .await
            .expect("failed to create server")
    }

    async fn client() -> Client {
        Client::new(nats().await)
    }

    async fn run_server_for_uds_cyclone() -> JoinHandle<Result<(), ServerError>> {
        tokio::spawn(server_for_uds_cyclone().await.run())
    }

    #[test(tokio::test)]
    async fn executes_simple_resolver_function() {
        run_server_for_uds_cyclone().await;
        let client = client().await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let mut parameters = HashMap::new();
        parameters.insert("value".to_string(), serde_json::json!("waffles_are_neat"));

        let request = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "upperCaseString".to_string(),
            parameters: Some(parameters),
            code_base64: base64::encode(
                "function upperCaseString(params) { return params.value.toUpperCase(); }",
            ),
        };

        let result = client
            .execute_resolver_function("veritech.function.resolver", tx, &request)
            .await
            .expect("failed to execute resolver function");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "1234");
                assert_eq!(success.data, serde_json::json!("WAFFLES_ARE_NEAT"));
                assert!(!success.unset);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn executes_simple_qualification_check() {
        run_server_for_uds_cyclone().await;
        let client = client().await;

        // Not going to check output here--we aren't emitting anything
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            while let Some(output) = rx.recv().await {
                info!("output: {:?}", output)
            }
        });

        let mut properties = HashMap::new();
        properties.insert("pkg".to_string(), serde_json::json!("cider"));

        let mut request = QualificationCheckRequest {
            execution_id: "5678".to_string(),
            handler: "check".to_string(),
            component: QualificationCheckComponent {
                name: "cider".to_string(),
                properties,
            },
            code_base64: base64::encode(indoc! {r#"
                function check(component) {
                    const name = component.name;
                    const pkg = component.properties?.pkg;

                    if (name == pkg) {
                        return { qualified: true };
                    } else {
                        return {
                            qualified: false,
                            message: "name '" + name + "' doesn't match pkg '" + pkg + "'",
                        };
                    }
                }
            "#}),
        };

        // Run a qualified check (i.e. qualification returns qualified == true)
        let result = client
            .execute_qualification_check("veritech.function.qualification", tx.clone(), &request)
            .await
            .expect("failed to execute qualification check");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "5678");
                assert!(success.qualified);
                assert_eq!(success.message, None);
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }

        request.execution_id = "9012".to_string();
        request.component.name = "emacs".to_string();

        // Now update the request to re-run an unqualified check (i.e. qualification returning
        // qualified == false)
        let result = client
            .execute_qualification_check("veritech.function.qualification", tx, &request)
            .await
            .expect("failed to execute qualification check");

        match result {
            FunctionResult::Success(success) => {
                assert_eq!(success.execution_id, "9012");
                assert!(!success.qualified);
                assert_eq!(
                    success.message,
                    Some("name 'emacs' doesn't match pkg 'cider'".to_string())
                );
            }
            FunctionResult::Failure(failure) => {
                panic!("function did not succeed and should have: {:?}", failure)
            }
        }
    }
}
