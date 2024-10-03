use std::{
    fmt,
    future::{Future, IntoFuture as _},
    io,
    sync::Arc,
};

use billing_events::{BillingEventsError, BillingEventsWorkQueue};
use data_warehouse_stream_client::DataWarehouseStreamClient;
use naxum::{
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        ack::AckLayer,
        matched_subject::{ForSubject, MatchedSubjectLayer},
        trace::TraceLayer,
    },
    response::{IntoResponse, Response},
    MessageHead, ServiceBuilder, ServiceExt as _, TowerServiceExt as _,
};
use si_data_nats::{async_nats, jetstream, NatsClient};
use si_data_nats::{
    async_nats::{
        error::Error as AsyncNatsError,
        jetstream::{
            consumer::{pull::Stream, StreamErrorKind},
            stream::ConsumerErrorKind,
        },
    },
    ConnectionMetadata,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::{
    app_state::{AppState, NoopAppState},
    config::Config,
    handlers,
};

const CONSUMER_NAME: &str = "forklift-server";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("async nats consumer error: {0}")]
    AsyncNatsConsumer(#[from] AsyncNatsError<ConsumerErrorKind>),
    #[error("async nats stream error: {0}")]
    AsyncNatsStream(#[from] AsyncNatsError<StreamErrorKind>),
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
    #[error("naxum error: {0}")]
    Naxum(#[source] io::Error),
    #[error("si data nats error: {0}")]
    SiDataNats(#[from] si_data_nats::Error),
}

type ServerResult<T> = Result<T, ServerError>;

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub(crate) struct ServerMetadata {
    #[allow(dead_code)]
    instance_id: String,
    #[allow(dead_code)]
    job_invoked_provider: &'static str,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    #[allow(dead_code)]
    pub(crate) fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Returns the job invoked provider.
    #[allow(dead_code)]
    pub(crate) fn job_invoked_provider(&self) -> &str {
        self.job_invoked_provider
    }
}

/// The forklift server instance with its inner naxum task.
pub struct Server {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
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
    /// Creates a forklift server with a running naxum task.
    #[instrument(name = "forklift.init.from_config", level = "info", skip_all)]
    pub async fn from_config(config: Config, token: CancellationToken) -> ServerResult<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: config.instance_id().into(),
            job_invoked_provider: "si",
        });

        let nats = Self::connect_to_nats(&config).await?;

        let connection_metadata = nats.metadata_clone();

        let incoming = {
            let queue = BillingEventsWorkQueue::get_or_create(jetstream::new(nats)).await?;
            let consumer_subject = queue.workspace_update_subject("*");
            queue
                .stream()
                .await?
                .create_consumer(Self::incoming_consumer_config(consumer_subject))
                .await?
                .messages()
                .await?
        };

        let inner = match config.data_warehouse_stream_name() {
            Some(stream_name) => {
                info!(%stream_name, "creating billing events app in data warehouse stream delivery mode...");
                let client = DataWarehouseStreamClient::new(stream_name).await;
                let state = AppState::new(client);
                Self::build_app(
                    state,
                    connection_metadata,
                    incoming,
                    config.concurrency_limit(),
                    token.clone(),
                )?
            }
            None => {
                info!("creating billing events app in no-op mode...");
                let state = NoopAppState::new();
                Self::build_noop_app(
                    state,
                    connection_metadata,
                    incoming,
                    config.concurrency_limit(),
                    token.clone(),
                )?
            }
        };

        Ok(Self {
            metadata,
            inner,
            shutdown_token: token,
        })
    }

    fn build_app(
        state: AppState,
        connection_metadata: Arc<ConnectionMetadata>,
        incoming: Stream,
        concurrency_limit: usize,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let app = ServiceBuilder::new()
            .layer(
                MatchedSubjectLayer::new().for_subject(ForkliftForSubject::with_prefix(
                    connection_metadata.subject_prefix(),
                )),
            )
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(AckLayer::new())
            .service(handlers::process_request.with_state(state))
            .map_response(Response::into_response);

        let inner =
            naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
                .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    /// Infallible wrapper around running the inner naxum task.
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running forklift main loop");
        }
    }

    /// Fallibly awaits the inner naxum task.
    pub async fn try_run(self) -> ServerResult<()> {
        self.inner.await.map_err(ServerError::Naxum)?;
        info!("forklift main loop shutdown complete");
        Ok(())
    }

    fn build_noop_app(
        state: NoopAppState,
        connection_metadata: Arc<ConnectionMetadata>,
        incoming: Stream,
        concurrency_limit: usize,
        token: CancellationToken,
    ) -> ServerResult<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
        let app = ServiceBuilder::new()
            .layer(
                MatchedSubjectLayer::new().for_subject(ForkliftForSubject::with_prefix(
                    connection_metadata.subject_prefix(),
                )),
            )
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(AckLayer::new())
            .service(handlers::process_request_noop.with_state(state))
            .map_response(Response::into_response);

        let inner =
            naxum::serve_with_incoming_limit(incoming, app.into_make_service(), concurrency_limit)
                .with_graceful_shutdown(naxum::wait_on_cancelled(token));

        Ok(Box::new(inner.into_future()))
    }

    #[instrument(name = "forklift.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &Config) -> ServerResult<NatsClient> {
        let client = NatsClient::new(config.nats()).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[inline]
    fn incoming_consumer_config(
        subject: impl Into<String>,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            filter_subject: subject.into(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
struct ForkliftForSubject {
    prefix: Option<()>,
}

impl ForkliftForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for ForkliftForSubject
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut naxum::Message<R>) {
        let mut parts = req.subject().split('.');

        match self.prefix {
            Some(_) => {
                if let (Some(prefix), Some(p1), Some(p2), Some(_workspace_id), None) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.:workspace_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (Some(p1), Some(p2), Some(_workspace_id), None) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                {
                    let matched = format!("{p1}.{p2}.:workspace_id");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
        }
    }
}
