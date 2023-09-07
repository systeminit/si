use dal::change_set_pointer::ChangeSetPointerError;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::{
    job::consumer::JobConsumerError, InitializationError, JobFailureError, JobQueueProcessor,
    NatsProcessor, TransactionsError,
};
use nats_subscriber::SubscriberError;

use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use si_rabbitmq::RabbitError;

use std::{io, path::Path, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{
        mpsc::{self},
        oneshot, watch,
    },
};

use veritech_client::{Client as VeritechClient, EncryptionKey, EncryptionKeyError};

use crate::Config;

mod change_set_loop;
mod management_loop;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("error when loading encryption key: {0}")]
    EncryptionKey(#[from] EncryptionKeyError),
    #[error(transparent)]
    Initialization(#[from] InitializationError),
    #[error(transparent)]
    JobConsumer(#[from] JobConsumerError),
    #[error(transparent)]
    JobFailure(#[from] Box<JobFailureError>),
    #[error("missing management message contents")]
    MissingManagementMessageContents,
    #[error("missing management message \"reply_to\" field")]
    MissingManagementMessageReplyTo,
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error("rabbit error {0}")]
    Rabbit(#[from] RabbitError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

impl From<PgPoolError> for ServerError {
    fn from(e: PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}

impl From<JobFailureError> for ServerError {
    fn from(e: JobFailureError) -> Self {
        Self::JobFailure(Box::new(e))
    }
}

impl From<TransactionsError> for ServerError {
    fn from(e: TransactionsError) -> Self {
        Self::Transactions(Box::new(e))
    }
}

type ServerResult<T> = Result<T, ServerError>;

/// The [`Server`] for managing rebaser tasks.
#[allow(missing_debug_implementations)]
pub struct Server {
    encryption_key: Arc<EncryptionKey>,
    nats: NatsClient,
    pg_pool: PgPool,
    veritech: VeritechClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    /// An internal shutdown watch receiver handle which can be provided to internal tasks which
    /// want to be notified when a shutdown event is in progress.
    shutdown_watch_rx: watch::Receiver<()>,
    /// An external shutdown sender handle which can be handed out to external callers who wish to
    /// trigger a server shutdown at will.
    external_shutdown_tx: mpsc::Sender<ShutdownSource>,
    /// An internal graceful shutdown receiver handle which the server's main thread uses to stop
    /// accepting work when a shutdown event is in progress.
    graceful_shutdown_rx: oneshot::Receiver<()>,
    /// If enabled, re-create the RabbitMQ Stream. If disabled, create the Stream if it does not
    /// exist.
    recreate_management_stream: bool,
}

impl Server {
    /// Build a [`Server`] from a given [`Config`].
    #[instrument(name = "rebaser.init.from_config", skip_all)]
    pub async fn from_config(config: Config) -> ServerResult<Self> {
        dal::init()?;

        let encryption_key =
            Self::load_encryption_key(config.cyclone_encryption_key_path()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone());

        Self::from_services(
            encryption_key,
            nats,
            pg_pool,
            veritech,
            job_processor,
            config.recreate_management_stream(),
        )
    }

    /// Build a [`Server`] from information provided via companion services.
    #[instrument(name = "rebaser.init.from_services", skip_all)]
    pub fn from_services(
        encryption_key: Arc<EncryptionKey>,
        nats: NatsClient,
        pg_pool: PgPool,
        veritech: VeritechClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        recreate_management_stream: bool,
    ) -> ServerResult<Self> {
        // An mpsc channel which can be used to externally shut down the server.
        let (external_shutdown_tx, external_shutdown_rx) = mpsc::channel(4);
        // A watch channel used to notify internal parts of the server that a shutdown event is in
        // progress. The value passed along is irrelevant--we only care that the event was
        // triggered and react accordingly.
        let (shutdown_watch_tx, shutdown_watch_rx) = watch::channel(());

        dal::init()?;

        let graceful_shutdown_rx =
            prepare_graceful_shutdown(external_shutdown_rx, shutdown_watch_tx)?;

        Ok(Server {
            recreate_management_stream,
            pg_pool,
            nats,
            veritech,
            encryption_key,
            job_processor,
            shutdown_watch_rx,
            external_shutdown_tx,
            graceful_shutdown_rx,
        })
    }

    /// The primary function for running the server. This should be called when deciding to run
    /// the server as a task, in a standalone binary, etc.
    pub async fn run(self) -> ServerResult<()> {
        management_loop::management_loop_infallible_wrapper(
            self.recreate_management_stream,
            self.pg_pool,
            self.nats,
            self.veritech,
            self.job_processor,
            self.encryption_key,
            self.shutdown_watch_rx,
        )
        .await;

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

    #[instrument(name = "rebaser.init.load_encryption_key", skip_all)]
    async fn load_encryption_key(path: impl AsRef<Path>) -> ServerResult<Arc<EncryptionKey>> {
        Ok(Arc::new(EncryptionKey::load(path).await?))
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
    shutdown_watch_tx: watch::Sender<()>,
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
            shutdown_watch_tx: watch::Sender<()>,
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
