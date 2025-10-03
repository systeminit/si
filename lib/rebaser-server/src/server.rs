use core::fmt;
use std::{
    future::{
        Future,
        IntoFuture,
    },
    io,
    sync::Arc,
    time::Duration,
};

use dal::{
    DalContext,
    DalLayerDb,
    DedicatedExecutor,
    JetstreamStreams,
    JobQueueProcessor,
    NatsProcessor,
    ServicesContext,
    feature_flags::FeatureFlagService,
};
use edda_client::EddaClient;
use nats_dead_letter_queue::DeadLetterQueue;
use naxum::{
    Message,
    MessageHead,
    ServiceBuilder,
    ServiceExt as _,
    TowerServiceExt as _,
    extract::MatchedSubject,
    handler::Handler as _,
    middleware::{
        ack::AckLayer,
        matched_subject::{
            ForSubject,
            MatchedSubjectLayer,
        },
        trace::{
            OnRequest,
            TraceLayer,
        },
    },
    response::{
        IntoResponse,
        Response,
    },
};
use pinga_client::PingaClient;
use rebaser_client::RebaserClient;
use rebaser_core::nats;
use si_crypto::{
    SymmetricCryptoService,
    SymmetricCryptoServiceConfig,
    VeritechCryptoConfig,
    VeritechEncryptionKey,
};
use si_data_nats::{
    NatsClient,
    NatsConfig,
    async_nats,
    jetstream,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use veritech_client::Client as VeritechClient;

use crate::{
    Config,
    Error,
    Features,
    Result,
    app_state::AppState,
    handlers,
};

const TASKS_CONSUMER_NAME: &str = "rebaser-tasks";

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    instance_id: String,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

/// A service which concurrently processes rebaser requests across multiple change sets.
///
/// Each change set is processed with a dedicated micro service (a [`ChangeSetRequestsTask`]) which
/// consumes a work queue of rebaser requests. The work queue semantics allows multiple Rebaser
/// server instances to run and share work while collectively processing one message at a time,
/// thus preserving the total order of requests within the scope of a chanage set.
///
/// An activity stream of all rebaser requests is processed in the main loop to determine if new
/// change set tasks should be spawned.
pub struct Server {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    server_tracker: TaskTracker,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl Server {
    /// Creates a runnable [`Server`] from configuration.
    #[instrument(name = "rebaser.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        shutdown_token: CancellationToken,
        layer_db_tracker: &TaskTracker,
        layer_db_token: CancellationToken,
    ) -> Result<Self> {
        dal::init()?;

        let encryption_key = Self::load_encryption_key(config.crypto().clone()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let jetstream_streams = JetstreamStreams::new(nats.clone()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let rebaser = Self::create_rebaser_client(nats.clone()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone()).await?;
        let symmetric_crypto_service =
            Self::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;
        let compute_executor = Self::create_compute_executor()?;

        let (layer_db, layer_db_graceful_shutdown) = DalLayerDb::from_config(
            config.layer_db_config().clone(),
            compute_executor.clone(),
            layer_db_token.clone(),
        )
        .await?;
        layer_db_tracker.spawn(layer_db_graceful_shutdown.into_future());

        let services_context = ServicesContext::new(
            pg_pool,
            nats.clone(),
            jetstream_streams,
            job_processor,
            rebaser,
            veritech.clone(),
            encryption_key,
            None,
            None,
            symmetric_crypto_service,
            layer_db,
            FeatureFlagService::default(),
            compute_executor,
        );

        Self::from_services(
            config.instance_id().to_string(),
            config.concurrency_limit(),
            services_context,
            config.quiescent_period(),
            shutdown_token,
            config.features(),
            config.snapshot_eviction_grace_period(),
        )
        .await
    }

    /// Creates a runnable [`Server`] from pre-configured and pre-created services.
    #[instrument(name = "rebaser.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        instance_id: impl Into<String>,
        concurrency_limit: Option<usize>,
        services_context: ServicesContext,
        quiescent_period: Duration,
        shutdown_token: CancellationToken,
        features: Features,
        snapshot_eviction_grace_period: Duration,
    ) -> Result<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: instance_id.into(),
        });

        let nats = services_context.nats_conn().clone();
        let context = jetstream::new(nats.clone());

        let prefix = nats.metadata().subject_prefix().map(|s| s.to_owned());

        let connection_metadata = nats.metadata_clone();

        let tasks = nats::rebaser_tasks_jetstream_stream(&context)
            .await?
            .create_consumer(Self::rebaser_tasks_consumer_config(prefix.as_deref()))
            .await?
            .messages()
            .await?;

        let requests_stream = nats::rebaser_requests_jetstream_stream(&context).await?;

        let dead_letter_queue = DeadLetterQueue::create_stream(context.clone()).await?;

        let pinga = PingaClient::new(nats.clone()).await?;

        let edda = EddaClient::new(nats.clone()).await?;

        let ctx_builder = DalContext::builder(services_context, false);

        let server_tracker = TaskTracker::new();
        let state = AppState::new(
            metadata.clone(),
            nats,
            pinga,
            edda,
            requests_stream,
            dead_letter_queue,
            ctx_builder,
            quiescent_period,
            shutdown_token.clone(),
            server_tracker.clone(),
            features,
            snapshot_eviction_grace_period,
        );

        let app = ServiceBuilder::new()
            .layer(
                MatchedSubjectLayer::new()
                    .for_subject(RebaserTasksForSubject::with_prefix(prefix.as_deref())),
            )
            .layer(
                TraceLayer::new()
                    .make_span_with(
                        telemetry_nats::NatsMakeSpan::builder(connection_metadata).build(),
                    )
                    .on_request(RebaserOnRequest)
                    .on_response(telemetry_nats::NatsOnResponse::new()),
            )
            .layer(AckLayer::new())
            .service(handlers::default.with_state(state))
            .map_response(Response::into_response);

        let inner = match concurrency_limit {
            Some(concurrency_limit) => Box::new(
                naxum::serve_with_incoming_limit(tasks, app.into_make_service(), concurrency_limit)
                    .with_graceful_shutdown(naxum::wait_on_cancelled(shutdown_token.clone()))
                    .into_future(),
            ),
            None => Box::new(
                naxum::serve(tasks, app.into_make_service())
                    .with_graceful_shutdown(naxum::wait_on_cancelled(shutdown_token.clone()))
                    .into_future(),
            ),
        };

        Ok(Self {
            metadata,
            inner,
            server_tracker,
        })
    }

    /// Runs the service to completion or until the first internal error is encountered.
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running rebaser main loop");
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    pub async fn try_run(self) -> Result<()> {
        self.inner.await.map_err(Error::Naxum)?;
        info!("rebaser inner loop exited, now shutting down the server tracker's tasks");
        self.server_tracker.close();
        self.server_tracker.wait().await;
        info!("rebaser main loop shutdown complete");
        Ok(())
    }

    #[inline]
    fn rebaser_tasks_consumer_config(
        subject_prefix: Option<&str>,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(TASKS_CONSUMER_NAME.to_owned()),
            filter_subject: nats::subject::tasks_incoming(subject_prefix).to_string(),
            ..Default::default()
        }
    }

    #[instrument(name = "rebaser.init.load_encryption_key", level = "info", skip_all)]
    async fn load_encryption_key(
        crypto_config: VeritechCryptoConfig,
    ) -> Result<Arc<VeritechEncryptionKey>> {
        Ok(Arc::new(
            VeritechEncryptionKey::from_config(crypto_config)
                .await
                .map_err(Error::CycloneEncryptionKey)?,
        ))
    }

    #[instrument(name = "rebaser.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "rebaser.init.create_pg_pool", level = "info", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config)
            .await
            .map_err(Error::dal_pg_pool)?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "rebaser.init.create_rebaser_client", level = "info", skip_all)]
    async fn create_rebaser_client(nats: NatsClient) -> Result<RebaserClient> {
        let client = RebaserClient::new(nats).await?;
        debug!("successfully initialized the rebaser client");
        Ok(client)
    }

    #[instrument(name = "rebaser.init.create_veritech_client", level = "info", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "rebaser.init.create_job_processor", level = "info", skip_all)]
    async fn create_job_processor(
        nats: NatsClient,
    ) -> Result<Box<dyn JobQueueProcessor + Send + Sync>> {
        Ok(Box::new(NatsProcessor::new(nats).await?) as Box<dyn JobQueueProcessor + Send + Sync>)
    }

    #[instrument(
        name = "rebaser.init.create_symmetric_crypto_service",
        level = "info",
        skip_all
    )]
    async fn create_symmetric_crypto_service(
        config: &SymmetricCryptoServiceConfig,
    ) -> Result<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "rebaser.init.create_compute_executor",
        level = "info",
        skip_all
    )]
    fn create_compute_executor() -> Result<DedicatedExecutor> {
        dal::compute_executor("rebaser").map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
struct RebaserOnRequest;

impl<R> OnRequest<R> for RebaserOnRequest
where
    R: MessageHead,
{
    fn on_request(&mut self, req: &Message<R>, _span: &Span) {
        debug!(task = req.subject().as_str(), "starting task");
        metric!(counter.change_set_processor_task.change_set_task = 1);
    }
}

#[derive(Clone, Debug)]
struct RebaserTasksForSubject {
    prefix: Option<()>,
}

impl RebaserTasksForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for RebaserTasksForSubject
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut naxum::Message<R>) {
        let mut parts = req.subject().split('.');

        match self.prefix {
            Some(_) => {
                if let (
                    Some(prefix),
                    Some(p1),
                    Some(p2),
                    Some(_workspace_id),
                    Some(_change_set_id),
                    Some(kind),
                    None,
                ) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{prefix}.{p1}.{p2}.:workspace_id.:change_set_id.{kind}");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
            None => {
                if let (
                    Some(p1),
                    Some(p2),
                    Some(_workspace_id),
                    Some(_change_set_id),
                    Some(kind),
                    None,
                ) = (
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                    parts.next(),
                ) {
                    let matched = format!("{p1}.{p2}.:workspace_id.:change_set_id.{kind}");
                    req.extensions_mut().insert(MatchedSubject::from(matched));
                };
            }
        }
    }
}
