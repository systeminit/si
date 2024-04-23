use std::{future::IntoFuture, io, sync::Arc};

use dal::{DalLayerDb, InitializationError, JobQueueProcessor, NatsProcessor};
use si_crypto::{CryptoConfig, SymmetricCryptoServiceConfig};
use si_crypto::{SymmetricCryptoError, SymmetricCryptoService};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use si_layer_cache::error::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{
        mpsc::{self},
        oneshot,
    },
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use veritech_client::{Client as VeritechClient, CycloneEncryptionKey, CycloneEncryptionKeyError};

use crate::server::core_loop::CoreLoopSetupError;
use crate::Config;

mod core_loop;
mod rebase;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("core loop setup error: {0}")]
    CoreLoopSetup(#[from] CoreLoopSetupError),
    #[error("error when loading encryption key: {0}")]
    CycloneEncryptionKey(#[from] CycloneEncryptionKeyError),
    #[error(transparent)]
    Initialization(#[from] InitializationError),
    #[error("layer cache error: {0}")]
    LayerCache(#[from] LayerDbError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
}

impl From<PgPoolError> for ServerError {
    fn from(e: PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}

type ServerResult<T> = Result<T, ServerError>;

/// The [`Server`] for managing rebaser tasks.
#[allow(missing_debug_implementations)]
pub struct Server {
    encryption_key: Arc<CycloneEncryptionKey>,
    nats: NatsClient,
    pg_pool: PgPool,
    veritech: VeritechClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    symmetric_crypto_service: SymmetricCryptoService,
    /// An internal shutdown watch receiver handle which can be provided to internal tasks which
    /// want to be notified when a shutdown event is in progress.
    shutdown_watch_rx: oneshot::Receiver<()>,
    /// An external shutdown sender handle which can be handed out to external callers who wish to
    /// trigger a server shutdown at will.
    external_shutdown_tx: mpsc::Sender<ShutdownSource>,
    /// An internal graceful shutdown receiver handle which the server's main thread uses to stop
    /// accepting work when a shutdown event is in progress.
    graceful_shutdown_rx: oneshot::Receiver<()>,
    /// The layer db
    layer_db: DalLayerDb,
}

impl Server {
    /// Build a [`Server`] from a given [`Config`].
    #[instrument(name = "rebaser.init.from_config", skip_all)]
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

        let mut pg_layer_db_pool = config.pg_pool().clone();
        pg_layer_db_pool.dbname = config.layer_cache_pg_dbname().to_string();

        let (layer_db, layer_db_graceful_shutdown) = DalLayerDb::initialize(
            config.layer_cache_disk_path(),
            PgPool::new(&pg_layer_db_pool).await?,
            nats.clone(),
            token,
        )
        .await?;
        tracker.spawn(layer_db_graceful_shutdown.into_future());

        Self::from_services(
            encryption_key,
            nats,
            pg_pool,
            veritech,
            job_processor,
            symmetric_crypto_service,
            layer_db,
        )
    }

    /// Build a [`Server`] from information provided via companion services.
    #[allow(clippy::too_many_arguments)]
    #[instrument(name = "rebaser.init.from_services", skip_all)]
    pub fn from_services(
        encryption_key: Arc<CycloneEncryptionKey>,
        nats: NatsClient,
        pg_pool: PgPool,
        veritech: VeritechClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        symmetric_crypto_service: SymmetricCryptoService,
        layer_db: DalLayerDb,
    ) -> ServerResult<Self> {
        // An mpsc channel which can be used to externally shut down the server.
        let (external_shutdown_tx, external_shutdown_rx) = mpsc::channel(4);
        // A watch channel used to notify internal parts of the server that a shutdown event is in
        // progress. The value passed along is irrelevant--we only care that the event was
        // triggered and react accordingly.
        let (shutdown_watch_tx, shutdown_watch_rx) = oneshot::channel();

        dal::init()?;

        let graceful_shutdown_rx =
            prepare_graceful_shutdown(external_shutdown_rx, shutdown_watch_tx)?;

        Ok(Server {
            pg_pool,
            nats,
            veritech,
            encryption_key,
            job_processor,
            symmetric_crypto_service,
            shutdown_watch_rx,
            external_shutdown_tx,
            graceful_shutdown_rx,
            layer_db,
        })
    }

    /// The primary function for running the server. This should be called when deciding to run
    /// the server as a task, in a standalone binary, etc.
    pub async fn run(self) -> ServerResult<()> {
        core_loop::setup_and_run_core_loop(
            self.pg_pool,
            self.nats,
            self.veritech,
            self.job_processor,
            self.symmetric_crypto_service,
            self.encryption_key,
            self.shutdown_watch_rx,
            self.layer_db,
        )
        .await?;

        let _ = self.graceful_shutdown_rx.await;
        info!("received and processed graceful shutdown, terminating server instance");

        Ok(())
    }

    /// Gets a [`ShutdownHandle`](ServerShutdownHandle) that can externally or on demand trigger the server's shutdown
    /// process.
    pub fn shutdown_handle(&self) -> ServerShutdownHandle {
        ServerShutdownHandle {
            shutdown_tx: self.external_shutdown_tx.clone(),
        }
    }

    #[instrument(name = "gobbler.init.load_encryption_key", skip_all)]
    async fn load_encryption_key(
        crypto_config: CryptoConfig,
    ) -> ServerResult<Arc<CycloneEncryptionKey>> {
        Ok(Arc::new(
            CycloneEncryptionKey::from_config(crypto_config).await?,
        ))
    }

    #[instrument(name = "rebaser.init.connect_to_nats", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "rebaser.init.create_pg_pool", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> ServerResult<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "rebaser.init.create_veritech_client", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "rebaser.init.create_job_processor", skip_all)]
    fn create_job_processor(nats: NatsClient) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(NatsProcessor::new(nats)) as Box<dyn JobQueueProcessor + Send + Sync>
    }

    #[instrument(name = "pinga.init.create_symmetric_crypto_service", skip_all)]
    async fn create_symmetric_crypto_service(
        config: &SymmetricCryptoServiceConfig,
    ) -> ServerResult<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
    }
}

#[allow(missing_docs, missing_debug_implementations)]
pub struct ServerShutdownHandle {
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl ServerShutdownHandle {
    /// Perform server shutdown with the handle.
    pub async fn shutdown(self) {
        if let Err(err) = self.shutdown_tx.send(ShutdownSource::Handle).await {
            warn!(error = ?err, "shutdown tx returned error, receiver is likely already closed");
        }
    }
}

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
enum ShutdownSource {
    Handle,
}

impl Default for ShutdownSource {
    fn default() -> Self {
        Self::Handle
    }
}

fn prepare_graceful_shutdown(
    mut external_shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_watch_tx: oneshot::Sender<()>,
) -> ServerResult<oneshot::Receiver<()>> {
    // A oneshot channel signaling the start of a graceful shutdown. Receivers can use this to
    // perform an clean/graceful shutdown work that needs to happen to preserve server integrity.
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    // A stream of `SIGTERM` signals, emitted as the process receives them.
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(
            graceful_shutdown_tx: oneshot::Sender<()>,
            shutdown_watch_tx: oneshot::Sender<()>,
        ) {
            // Send shutdown to all long running subscriptions, so they can cleanly terminate
            if shutdown_watch_tx.send(()).is_err() {
                error!("all watch shutdown receivers have already been dropped");
            }
            // Send graceful shutdown to main server thread which stops it from accepting requests.
            // We'll do this step last so as to let all subscriptions have a chance to shutdown.
            if graceful_shutdown_tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
        }

        info!("spawned graceful shutdown handler");

        tokio::select! {
            _ = sigterm_stream.recv() => {
                info!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_watch_tx);
            }
            source = external_shutdown_rx.recv() => {
                info!(
                    "received external shutdown, performing graceful shutdown; source={:?}",
                    source,
                );
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_watch_tx);
            }
            else => {
                // All other arms are closed, nothing left to do but return
                trace!("returning from graceful shutdown with all select arms closed");
            }
        };
    });

    Ok(graceful_shutdown_rx)
}
