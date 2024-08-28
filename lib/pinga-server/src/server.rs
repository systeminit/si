use std::{
    fmt,
    future::{Future, IntoFuture as _},
    io,
    sync::Arc,
};

use dal::{
    feature_flags::FeatureFlagService, DalContext, JobQueueProcessor, NatsProcessor,
    ServicesContext,
};
use naxum::{
    handler::Handler as _,
    middleware::{
        ack::AckLayer,
        trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer},
    },
    ServiceExt as _,
};
use pinga_core::{pinga_work_queue, subject};
use si_crypto::{
    SymmetricCryptoService, SymmetricCryptoServiceConfig, VeritechCryptoConfig,
    VeritechEncryptionKey,
};
use si_data_nats::{async_nats, jetstream, NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use si_layer_cache::LayerDb;
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower::ServiceBuilder;
use veritech_client::Client as VeritechClient;

use crate::{app_state::AppState, handlers, Config, ServerError, ServerResult};

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
        tracker: TaskTracker,
    ) -> ServerResult<Self> {
        dal::init()?;

        let encryption_key = Self::load_encryption_key(config.crypto().clone()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone());
        let symmetric_crypto_service =
            Self::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

        let (layer_db, layer_db_graceful_shutdown) =
            LayerDb::from_config(config.layer_db_config().clone(), token.clone()).await?;
        tracker.spawn(layer_db_graceful_shutdown.into_future());

        let services_context = ServicesContext::new(
            pg_pool,
            nats.clone(),
            job_processor,
            veritech.clone(),
            encryption_key,
            None,
            None,
            symmetric_crypto_service,
            layer_db,
            FeatureFlagService::default(),
        );

        Self::from_services(
            config.instance_id().to_string(),
            config.concurrency_limit(),
            services_context,
            token,
        )
        .await
    }

    #[instrument(name = "pinga.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        instance_id: impl Into<String>,
        concurrency_limit: usize,
        services_context: ServicesContext,
        shutdown_token: CancellationToken,
    ) -> ServerResult<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: instance_id.into(),
            job_invoked_provider: "si",
        });

        // Take the *active* subject prefix from the connected NATS client
        let prefix = services_context
            .nats_conn()
            .metadata()
            .subject_prefix()
            .map(|s| s.to_owned());

        let context = jetstream::new(services_context.nats_conn().clone());

        let incoming = pinga_work_queue(&context, prefix.as_deref())
            .await?
            .create_consumer(Self::incoming_consumer_config(prefix.as_deref()))
            .await?
            .messages()
            .await?;

        let ctx_builder = DalContext::builder(services_context, false);

        let state = AppState::new(metadata.clone(), concurrency_limit, ctx_builder);

        let app = ServiceBuilder::new()
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

    #[instrument(name = "pinga.init.create_veritech_client", level = "info", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "pinga.init.create_job_processor", level = "info", skip_all)]
    fn create_job_processor(nats: NatsClient) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(NatsProcessor::new(nats)) as Box<dyn JobQueueProcessor + Send + Sync>
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
