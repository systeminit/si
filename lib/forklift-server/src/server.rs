use std::{
    fmt,
    future::Future,
    io,
    sync::Arc,
};

use audit_database::{
    AuditDatabaseContext,
    AuditDatabaseContextError,
};
use si_data_nats::{
    ConnectionMetadata,
    NatsClient,
    jetstream,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
};
use si_layer_cache::event::LayeredEventClient;
use snapshot_eviction::SnapshotEvictor;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;
use tokio_util::sync::CancellationToken;

use crate::config::Config;

mod app;

pub(crate) use app::AppSetupError;

const DURABLE_CONSUMER_NAME: &str = "forklift-server";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("app setup error: {0}")]
    AppSetup(#[from] AppSetupError),
    #[error("audit database context error: {0}")]
    AuditDatabaseContext(#[from] AuditDatabaseContextError),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("naxum error: {0}")]
    Naxum(#[source] io::Error),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("si data nats error: {0}")]
    SiDataNats(#[from] si_data_nats::Error),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("snapshot eviction error: {0}")]
    SnapshotEviction(#[from] snapshot_eviction::SnapshotEvictionError),
}

type Result<T> = std::result::Result<T, ServerError>;

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
    shutdown_token: CancellationToken,
    // TODO(nick): remove option once this is working.
    inner_audit_logs: Option<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>>,
    inner_billing_events: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
    snapshot_evictor: SnapshotEvictor,
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
    /// Creates a forklift server with a running naxum task from a given configuration.
    #[instrument(name = "forklift.init.from_config", level = "info", skip_all)]
    pub async fn from_config(config: Config, token: CancellationToken) -> Result<Self> {
        let nats = Self::connect_to_nats(&config).await?;
        let connection_metadata = nats.metadata_clone();
        let jetstream_context = jetstream::new(nats.clone());

        let audit_bag = if config.enable_audit_logs_app() {
            let insert_concurrency_limit = config.audit().insert_concurrency_limit;
            let audit_database_context = AuditDatabaseContext::from_config(config.audit()).await?;
            Some((audit_database_context, insert_concurrency_limit))
        } else {
            None
        };

        // Initialize pools for eviction task
        let si_db_pool = Self::create_si_db_pool(&config.snapshot_eviction().si_db).await?;
        let layer_cache_pool =
            Self::create_layer_cache_pool(&config.snapshot_eviction().layer_cache_pg).await?;

        // Create LayeredEventClient for eviction
        let layered_event_client =
            Self::create_layered_event_client(&nats, config.instance_id(), &jetstream_context)?;

        // Validate and clamp eviction config
        let mut eviction_config = config.snapshot_eviction().clone();
        eviction_config.validate_and_clamp();

        Self::from_services(
            connection_metadata,
            jetstream_context,
            config.instance_id(),
            config.concurrency_limit(),
            audit_bag,
            config.data_warehouse_stream_name(),
            si_db_pool,
            layer_cache_pool,
            layered_event_client,
            eviction_config,
            token,
        )
        .await
    }

    /// Creates a forklift server with a running naxum task with running services.
    #[allow(clippy::too_many_arguments)]
    #[instrument(name = "forklift.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        connection_metadata: Arc<ConnectionMetadata>,
        jetstream_context: jetstream::Context,
        instance_id: &str,
        concurrency_limit: usize,
        audit_bag: Option<(AuditDatabaseContext, usize)>,
        data_warehouse_stream_name: Option<&str>,
        si_db_pool: PgPool,
        layer_cache_pool: PgPool,
        layered_event_client: LayeredEventClient,
        snapshot_eviction_config: snapshot_eviction::SnapshotEvictionConfig,
        token: CancellationToken,
    ) -> Result<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: instance_id.into(),
            job_invoked_provider: "si",
        });

        let inner_audit_logs =
            if let Some((audit_database_context, insert_concurrency_limit)) = audit_bag {
                Some(
                    app::audit_logs(
                        jetstream_context.clone(),
                        DURABLE_CONSUMER_NAME.to_string(),
                        connection_metadata.clone(),
                        audit_database_context,
                        insert_concurrency_limit,
                        token.clone(),
                    )
                    .await?,
                )
            } else {
                None
            };
        let inner_billing_events = app::billing_events(
            jetstream_context,
            DURABLE_CONSUMER_NAME.to_string(),
            connection_metadata,
            concurrency_limit,
            data_warehouse_stream_name,
            token.clone(),
        )
        .await?;

        // Create snapshot evictor
        let snapshot_evictor = SnapshotEvictor::new(
            si_db_pool,
            layer_cache_pool,
            layered_event_client,
            snapshot_eviction_config,
        );

        Ok(Self {
            metadata,
            inner_audit_logs,
            inner_billing_events,
            snapshot_evictor,
            shutdown_token: token,
        })
    }

    /// Infallible wrapper around running the inner naxum task(s).
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running forklift main loop");
        }
    }

    /// Fallibly awaits the inner naxum task(s).
    pub async fn try_run(self) -> Result<()> {
        // Extract fields to avoid partial move issues
        let snapshot_evictor = self.snapshot_evictor;
        let inner_audit_logs = self.inner_audit_logs;
        let inner_billing_events = self.inner_billing_events;
        let shutdown_token = self.shutdown_token;

        // Spawn snapshot eviction background task
        let eviction_shutdown = shutdown_token.clone();
        let eviction_task =
            tokio::spawn(async move { snapshot_evictor.run(eviction_shutdown).await });

        info!("Snapshot eviction task spawned");

        // Run existing app tasks
        let result = match inner_audit_logs {
            Some(inner_audit_logs) => {
                info!("running three apps: audit logs, billing events, and snapshot eviction");
                let (eviction_result, audit_result, billing_result) = futures::join!(
                    eviction_task,
                    tokio::spawn(inner_audit_logs),
                    tokio::spawn(inner_billing_events)
                );

                // Check eviction task
                eviction_result??;

                // Check existing tasks
                audit_result?.map_err(ServerError::Naxum)?;
                billing_result?.map_err(ServerError::Naxum)?;
                Ok(())
            }
            None => {
                info!("running two apps: billing events and snapshot eviction");
                let (eviction_result, billing_result) =
                    futures::join!(eviction_task, tokio::spawn(inner_billing_events));

                // Check eviction task
                eviction_result??;

                // Check billing task
                billing_result?.map_err(ServerError::Naxum)?;
                Ok(())
            }
        };

        info!("forklift main loop shutdown complete");
        result
    }

    #[instrument(name = "forklift.init.create_si_db_pool", level = "info", skip_all)]
    async fn create_si_db_pool(config: &PgPoolConfig) -> Result<PgPool> {
        let mut pool_config = config.clone();
        // Minimal pool for single eviction task
        pool_config.pool_max_size = 2;

        let pool = PgPool::new(&pool_config).await?;
        debug!("si-db pool initialized for eviction (pool_max_size=2)");
        Ok(pool)
    }

    #[instrument(
        name = "forklift.init.create_layer_cache_pool",
        level = "info",
        skip_all
    )]
    async fn create_layer_cache_pool(config: &PgPoolConfig) -> Result<PgPool> {
        let mut pool_config = config.clone();
        // Minimal pool for single eviction task
        pool_config.pool_max_size = 2;

        let pool = PgPool::new(&pool_config).await?;
        debug!("layer-cache pool initialized for eviction (pool_max_size=2)");
        Ok(pool)
    }

    #[instrument(
        name = "forklift.init.create_layered_event_client",
        level = "info",
        skip_all
    )]
    fn create_layered_event_client(
        nats: &NatsClient,
        instance_id: &str,
        jetstream_context: &jetstream::Context,
    ) -> Result<LayeredEventClient> {
        let instance_id_ulid = ulid::Ulid::from_string(instance_id)?;

        let client = LayeredEventClient::new(
            nats.metadata().subject_prefix().map(|s| s.to_owned()),
            instance_id_ulid,
            jetstream_context.clone(),
        );

        debug!("LayeredEventClient created for snapshot eviction");
        Ok(client)
    }

    #[instrument(name = "forklift.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &Config) -> Result<NatsClient> {
        let client = NatsClient::new(config.nats()).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }
}
