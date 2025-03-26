use std::{
    fmt,
    future::{Future, IntoFuture},
    io,
    sync::Arc,
};

use dal::{
    feature_flags::FeatureFlagService, DalLayerDb, DedicatedExecutor, JetstreamStreams,
    JobQueueProcessor, NatsProcessor, ServicesContext,
};
use rebaser_client::RebaserClient;
use si_crypto::{
    SymmetricCryptoService, SymmetricCryptoServiceConfig, VeritechCryptoConfig,
    VeritechEncryptionKey,
};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use telemetry::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use veritech_client::Client as VeritechClient;

use crate::{Config, Error, Result};

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
    #[instrument(name = "edda.init.from_config", level = "info", skip_all)]
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
        let job_processor = Self::create_job_processor(nats.clone());
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
            shutdown_token,
        )
        .await
    }

    /// Creates a runnable [`Server`] from pre-configured and pre-created services.
    #[instrument(name = "edda.init.from_services", level = "info", skip_all)]
    pub async fn from_services(
        instance_id: impl Into<String>,
        concurrency_limit: Option<usize>,
        services_context: ServicesContext,
        shutdown_token: CancellationToken,
    ) -> Result<Self> {
        todo!()
    }

    /// Runs the service to completion or until the first internal error is encountered.
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running edda main loop");
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    pub async fn try_run(self) -> Result<()> {
        self.inner.await.map_err(Error::Naxum)?;
        info!("edda inner loop exited, now shutting down the server tracker's tasks");
        self.server_tracker.close();
        self.server_tracker.wait().await;
        info!("edda main loop shutdown complete");
        Ok(())
    }

    #[instrument(name = "edda.init.load_encryption_key", level = "info", skip_all)]
    async fn load_encryption_key(
        crypto_config: VeritechCryptoConfig,
    ) -> Result<Arc<VeritechEncryptionKey>> {
        Ok(Arc::new(
            VeritechEncryptionKey::from_config(crypto_config)
                .await
                .map_err(Error::CycloneEncryptionKey)?,
        ))
    }

    #[instrument(name = "edda.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "edda.init.create_pg_pool", level = "info", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config)
            .await
            .map_err(Error::dal_pg_pool)?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "edda.init.create_rebaser_client", level = "info", skip_all)]
    async fn create_rebaser_client(nats: NatsClient) -> Result<RebaserClient> {
        let client = RebaserClient::new(nats).await?;
        debug!("successfully initialized the edda client");
        Ok(client)
    }

    #[instrument(name = "edda.init.create_veritech_client", level = "info", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "edda.init.create_job_processor", level = "info", skip_all)]
    fn create_job_processor(nats: NatsClient) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(NatsProcessor::new(nats)) as Box<dyn JobQueueProcessor + Send + Sync>
    }

    #[instrument(
        name = "edda.init.create_symmetric_crypto_service",
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

    #[instrument(name = "edda.init.create_compute_executor", level = "info", skip_all)]
    fn create_compute_executor() -> Result<DedicatedExecutor> {
        dal::compute_executor("edda").map_err(Into::into)
    }
}
