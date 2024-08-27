use futures::{join, Future, StreamExt};
use naxum::handler::Handler as _;
use naxum::middleware::ack::AckLayer;
use naxum::middleware::trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer};
use naxum::ServiceBuilder;
use naxum::ServiceExt as _;

use si_crypto::{VeritechDecryptionKey, VeritechDecryptionKeyError};
use si_data_nats::{async_nats, jetstream, NatsClient, NatsError, Subject, Subscriber};
use si_pool_noodle::{
    instance::cyclone::{LocalUdsInstance, LocalUdsInstanceSpec},
    PoolNoodle, Spec,
};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::future::IntoFuture;
use std::time::Duration;
use std::{io, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinError;
use tokio_util::sync::CancellationToken;
use veritech_core::{subject, veritech_work_queue, ExecutionId};

use crate::app_state::{AppState, KillAppState};
use crate::handlers;
use crate::{config::CycloneSpec, Config};

const CONSUMER_NAME: &str = "veritech-server";

const DEFAULT_CONCURRENCY_LIMIT: usize = 1000;
const DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT: Duration = Duration::from_secs(35 * 60);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("cyclone pool error: {0}")]
    CyclonePool(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("cyclone spec setup error: {0}")]
    CycloneSetupError(#[source] Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("jetstream consumer error: {0}")]
    JetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
    #[error("jetstream consumer stream error: {0}")]
    JetStreamConsumerStream(#[from] async_nats::jetstream::consumer::StreamError),
    #[error("jetstream create stream error: {0}")]
    JetStreamCreateStreamError(#[from] async_nats::jetstream::context::CreateStreamError),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("failed to initialize a nats client: {0}")]
    NatsClient(#[source] NatsError),
    #[error("failed to subscribe to nats subject ({0}): {1}")]
    NatsSubscribe(Subject, #[source] NatsError),
    #[error("naxum error: {0}")]
    Naxum(#[source] io::Error),
    #[error("veritech decryption key error: {0}")]
    VeritechDecryptionKey(#[from] VeritechDecryptionKeyError),
    #[error("wrong cyclone spec type for {0} spec: {1:?}")]
    WrongCycloneSpec(&'static str, Box<CycloneSpec>),
}

type ServerResult<T> = Result<T, ServerError>;

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    #[allow(unused)]
    instance_id: String,
    #[allow(unused)]
    job_invoked_provider: &'static str,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    #[allow(unused)]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Returns the job invoked provider.
    #[allow(unused)]
    pub fn job_invoked_provider(&self) -> &str {
        self.job_invoked_provider
    }
}

pub struct Server {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    kill_inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    shutdown_token: CancellationToken,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("metadata", &self.metadata)
            .field("shutdown_token", &self.shutdown_token)
            .finish()
    }
}

impl Server {
    #[instrument(name = "veritech.init.cyclone.uds", level = "info", skip_all)]
    pub async fn for_cyclone_uds(config: Config, token: CancellationToken) -> ServerResult<Self> {
        match config.cyclone_spec() {
            CycloneSpec::LocalUds(spec) => {
                let nats = Self::connect_to_nats(&config).await?;

                let mut cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec> =
                    PoolNoodle::new(spec.pool_size.into(), spec.clone(), token.clone());

                spec.clone()
                    .setup()
                    .await
                    .map_err(|e| ServerError::CycloneSetupError(Box::new(e)))?;
                cyclone_pool
                    .start(config.healthcheck_pool())
                    .map_err(|e| ServerError::CyclonePool(Box::new(e)))?;

                let metadata = Arc::new(ServerMetadata {
                    instance_id: config.instance_id().into(),
                    job_invoked_provider: "si",
                });

                let decryption_key =
                    VeritechDecryptionKey::from_config(config.crypto().clone()).await?;

                let cyclone_client_execution_timeout =
                    match config.cyclone_client_execution_timeout() {
                        Some(timeout) => Duration::from_secs(timeout),
                        None => DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT,
                    };

                let kill_senders = Arc::new(Mutex::new(HashMap::new()));

                let inner_future = Self::build_app(
                    metadata.clone(),
                    cyclone_pool,
                    Arc::new(decryption_key),
                    cyclone_client_execution_timeout,
                    nats.clone(),
                    kill_senders.clone(),
                    token.clone(),
                )
                .await?;
                let kill_inner_future =
                    Self::build_kill_app(metadata.clone(), nats, kill_senders, token.clone())
                        .await?;

                Ok(Server {
                    metadata,
                    inner: inner_future,
                    kill_inner: kill_inner_future,
                    shutdown_token: token,
                })
            }
            wrong @ CycloneSpec::LocalHttp(_) => Err(ServerError::WrongCycloneSpec(
                "LocalUds",
                Box::new(wrong.clone()),
            )),
        }
    }

    #[instrument(name = "veritech.init.cyclone.http", level = "info", skip_all)]
    pub async fn for_cyclone_http(config: Config, _token: CancellationToken) -> ServerResult<Self> {
        match config.cyclone_spec() {
            CycloneSpec::LocalHttp(_spec) => {
                // TODO(fnichol): Hi there! Ultimately, the Veritech server should be able to work
                // with a LocalUds Cyclone backend and a LocalHttp version. But since this involves
                // threading through some generic types which themselves have trait
                // constraints--well we can add this back in the near future... Note that the
                // immediately prior state to this line change is roughly the starting point for
                // adding the types back. Good luck to us all.
                //
                // let nats = connect_to_nats(&config).await?;
                // let manager = Manager::new(spec.clone());
                // let cyclone_pool = Pool::builder(manager)
                //     .build()
                //     .map_err(|err| ServerError::CycloneSpec(Box::new(err)))?;

                // Ok(Server { nats, cyclone_pool })
                unimplemented!("get ready for a surprise!!")
            }
            wrong @ CycloneSpec::LocalUds(_) => Err(ServerError::WrongCycloneSpec(
                "LocalHttp",
                Box::new(wrong.clone()),
            )),
        }
    }

    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running veritech main loop");
        }
    }

    pub async fn try_run(self) -> ServerResult<()> {
        let (inner_result, kill_inner_result) =
            join!(tokio::spawn(self.inner), tokio::spawn(self.kill_inner));

        inner_result?.map_err(ServerError::Naxum)?;
        kill_inner_result?.map_err(ServerError::Naxum)?;

        info!("veritech main loop shutdown complete");
        Ok(())
    }

    async fn build_app(
        metadata: Arc<ServerMetadata>,
        cyclone_pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
        decryption_key: Arc<VeritechDecryptionKey>,
        cyclone_client_execution_timeout: Duration,
        nats: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let incoming = {
            // Take the *active* subject prefix from the connected NATS client
            let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());
            let context = jetstream::new(nats.clone());
            veritech_work_queue(&context, prefix.as_deref())
                .await?
                .create_consumer(Self::incoming_consumer_config(prefix.as_deref()))
                .await?
                .messages()
                .await?
        };

        let state = AppState::new(
            metadata,
            cyclone_pool,
            decryption_key,
            cyclone_client_execution_timeout,
            nats,
            kill_senders,
        );

        let app = ServiceBuilder::new()
            .concurrency_limit(DEFAULT_CONCURRENCY_LIMIT)
            .layer(
                TraceLayer::new()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(
                        naxum::middleware::trace::DefaultOnResponse::new().level(Level::TRACE),
                    ),
            )
            .layer(AckLayer::new())
            .service(handlers::process_request.with_state(state));

        let inner = naxum::serve(incoming, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    async fn build_kill_app(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        kill_senders: Arc<Mutex<HashMap<ExecutionId, oneshot::Sender<()>>>>,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let incoming = {
            let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());
            Self::kill_subscriber(&nats, prefix.as_deref())
                .await?
                .map(|msg| msg.into_parts().0)
                // Core NATS subscriptions are a stream of `Option<Message>` so we convert this into a
                // stream of `Option<Result<Message, Infallible>>`
                .map(Ok::<_, Infallible>)
        };

        let state = KillAppState::new(metadata, nats, kill_senders);

        let app = ServiceBuilder::new()
            .layer(
                TraceLayer::new()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_request(DefaultOnRequest::new().level(Level::TRACE))
                    .on_response(
                        naxum::middleware::trace::DefaultOnResponse::new().level(Level::TRACE),
                    ),
            )
            .service(handlers::process_kill_request.with_state(state));

        let inner = naxum::serve(incoming, app.into_make_service())
            .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    // NOTE(nick,fletcher): it's a little funky that the prefix is taken from the nats client, but we ask for both
    // here. We don't want users to forget the prefix, so being explicit is helpful, but maybe we can change this
    // in the future.
    async fn kill_subscriber(nats: &NatsClient, prefix: Option<&str>) -> ServerResult<Subscriber> {
        let subject = veritech_core::subject::veritech_kill_request(prefix);
        nats.subscribe(subject.clone())
            .await
            .map_err(|err| ServerError::NatsSubscribe(subject, err))
    }

    #[instrument(name = "veritech.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &Config) -> ServerResult<NatsClient> {
        let client = NatsClient::new(config.nats())
            .await
            .map_err(ServerError::NatsClient)?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[inline]
    fn incoming_consumer_config(
        subject_prefix: Option<&str>,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            filter_subject: subject::incoming(subject_prefix).to_string(),
            ..Default::default()
        }
    }
}
