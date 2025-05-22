use std::{
    fmt,
    future::{
        Future,
        IntoFuture as _,
    },
    io,
    sync::Arc,
};

use dal::{
    DalContext,
    DedicatedExecutor,
    JetstreamStreams,
    JobQueueProcessor,
    NatsProcessor,
    ServicesContext,
    feature_flags::FeatureFlagService,
};
use naxum::{
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
        trace::TraceLayer,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use pinga_core::nats::{
    pinga_work_queue,
    subject,
};
use rebaser_client::RebaserClient;
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
use si_layer_cache::LayerDb;
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use veritech_client::Client as VeritechClient;

use crate::{
    Config,
    ServerError,
    ServerResult,
    app_state::AppState,
    handlers,
};

const CONSUMER_NAME: &str = "pinga-server";

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    instance_id: String,
    job_invoked_provider: &'static str,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Returns the job invoked provider.
    pub fn job_invoked_provider(&self) -> &str {
        self.job_invoked_provider
    }
}

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
    #[instrument(name = "pinga.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        token: CancellationToken,
        layer_db_tracker: &TaskTracker,
        layer_db_token: CancellationToken,
    ) -> ServerResult<Self> {
        dal::init()?;

        let encryption_key = Self::load_encryption_key(config.crypto().clone()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let nats_streams = JetstreamStreams::new(nats.clone()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let rebaser = Self::create_rebaser_client(nats.clone()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone()).await?;
        let symmetric_crypto_service =
            Self::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;
        let compute_executor = Self::create_compute_executor()?;

        let (layer_db, layer_db_graceful_shutdown) = LayerDb::from_config(
            config.layer_db_config().clone(),
            compute_executor.clone(),
            layer_db_token,
        )
        .await?;
        layer_db_tracker.spawn(layer_db_graceful_shutdown.into_future());

        let services_context = ServicesContext::new(
            pg_pool,
            nats,
            nats_streams,
            job_processor,
            rebaser,
            veritech,
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
            config.max_deliver(),
            services_context,
            token,
        )
        .await
    }

    #[instrument(name = "pinga.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        instance_id: impl Into<String>,
        concurrency_limit: usize,
        max_deliver: i64,
        services_context: ServicesContext,
        shutdown_token: CancellationToken,
    ) -> ServerResult<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: instance_id.into(),
            job_invoked_provider: "si",
        });

        let connection_metadata = services_context.nats_conn().metadata_clone();

        // Take the *active* subject prefix from the connected NATS client
        let prefix = services_context
            .nats_conn()
            .metadata()
            .subject_prefix()
            .map(|s| s.to_owned());

        let nats = services_context.nats_conn().clone();
        let context = jetstream::new(nats.clone());

        let incoming = pinga_work_queue(&context)
            .await?
            .create_consumer(Self::incoming_consumer_config(
                prefix.as_deref(),
                max_deliver,
            ))
            .await?
            .messages()
            .await?;

        let ctx_builder = DalContext::builder(services_context, false);

        let state = AppState::new(metadata.clone(), concurrency_limit, nats, ctx_builder);

        let app = ServiceBuilder::new()
            .layer(
                MatchedSubjectLayer::new()
                    .for_subject(PingaForSubject::with_prefix(prefix.as_deref())),
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
                .with_graceful_shutdown(naxum::wait_on_cancelled(shutdown_token.clone()));

        metric!(monotonic_counter.pinga.concurrency.limit = concurrency_limit);
        Ok(Self {
            metadata,
            inner: Box::new(inner.into_future()),
            shutdown_token,
        })
    }

    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running pinga main loop");
        }
    }

    pub async fn try_run(self) -> ServerResult<()> {
        self.inner.await.map_err(ServerError::Naxum)?;
        info!("pinga main loop shutdown complete");
        Ok(())
    }

    #[instrument(name = "pinga.init.load_encryption_key", level = "info", skip_all)]
    async fn load_encryption_key(
        crypto_config: VeritechCryptoConfig,
    ) -> ServerResult<Arc<VeritechEncryptionKey>> {
        Ok(Arc::new(
            VeritechEncryptionKey::from_config(crypto_config).await?,
        ))
    }

    #[instrument(name = "pinga.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(nats_config)
            .await
            .map_err(ServerError::NatsClient)?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "pinga.init.create_pg_pool", level = "info", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> ServerResult<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "pinga.init.create_rebaser_client", level = "info", skip_all)]
    async fn create_rebaser_client(nats: NatsClient) -> ServerResult<RebaserClient> {
        let client = RebaserClient::new(nats).await?;
        debug!("successfully initialized the rebaser client");
        Ok(client)
    }

    #[instrument(name = "pinga.init.create_veritech_client", level = "info", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "pinga.init.create_job_processor", level = "info", skip_all)]
    async fn create_job_processor(
        nats: NatsClient,
    ) -> ServerResult<Box<dyn JobQueueProcessor + Send + Sync>> {
        Ok(Box::new(NatsProcessor::new(nats).await?) as Box<dyn JobQueueProcessor + Send + Sync>)
    }

    #[instrument(
        name = "pinga.init.create_symmetric_crypto_service",
        level = "info",
        skip_all
    )]
    async fn create_symmetric_crypto_service(
        config: &SymmetricCryptoServiceConfig,
    ) -> ServerResult<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "pinga.init.create_compute_executor", level = "info", skip_all)]
    fn create_compute_executor() -> ServerResult<DedicatedExecutor> {
        dal::compute_executor("pinga").map_err(Into::into)
    }

    #[inline]
    fn incoming_consumer_config(
        subject_prefix: Option<&str>,
        max_deliver: i64,
    ) -> async_nats::jetstream::consumer::pull::Config {
        async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            filter_subject: subject::incoming(subject_prefix).to_string(),
            // TODO(nick,fletcher): this should eventually be "1" and not be configurable.
            max_deliver,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
struct PingaForSubject {
    prefix: Option<()>,
}

impl PingaForSubject {
    fn with_prefix(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|_p| ()),
        }
    }
}

impl<R> ForSubject<R> for PingaForSubject
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
