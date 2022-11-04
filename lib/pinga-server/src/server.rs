use std::{
    any::TypeId, future::Future, io, panic::AssertUnwindSafe, path::Path, sync::Arc, time::Duration,
};

use dal::{
    job::{
        consumer::JobConsumerError,
        definition::{
            CodeGeneration, Confirmation, Confirmations, DependentValuesUpdate, FixesJob,
            Qualification, Qualifications, WorkflowRun,
        },
    },
    DalContext, DalContextBuilder, FaktoryProcessor, InitializationError, JobFailure,
    JobFailureError, JobQueueProcessor, ServicesContext, TransactionsError,
};
use futures::future::FutureExt;
use si_data_faktory::{BeatState, Client as FaktoryClient, FailConfig, FaktoryConfig};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{mpsc, oneshot, watch},
    task::JoinError,
};
use uuid::Uuid;
use veritech_client::{Client as VeritechClient, EncryptionKey, EncryptionKeyError};

use crate::Config;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    Initialization(#[from] InitializationError),
    #[error("error when loading encryption key: {0}")]
    EncryptionKey(#[from] EncryptionKeyError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
}

impl From<PgPoolError> for ServerError {
    fn from(e: PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server {
    encryption_key: Arc<EncryptionKey>,
    nats: NatsClient,
    pg_pool: PgPool,
    veritech: VeritechClient,
    faktory: FaktoryClient,
    /// An internal shutdown watch receiver handle which can be provided to internal tasks which
    /// want to be notified when a shutdown event is in progress.
    shutdown_watch_rx: watch::Receiver<()>,
    /// An external shutdown sender handle which can be handed out to external callers who wish to
    /// trigger a server shutdown at will.
    external_shutdown_tx: mpsc::Sender<ShutdownSource>,
    /// An internal graceful shutdown receiever handle which the server's main thread uses to stop
    /// accepting work when a shutdown event is in progress.
    graceful_shutdown_rx: oneshot::Receiver<()>,
}

impl Server {
    #[instrument(name = "pinga.init.from_config", skip_all)]
    pub async fn from_config(config: Config) -> Result<Self> {
        // An mpsc channel which can be used to externally shut down the server.
        let (external_shutdown_tx, external_shutdown_rx) = mpsc::channel(4);
        // A watch channel used to notify internal parts of the server that a shutdown event is in
        // progress. The value passed along is irrelevant--we only care that the event was
        // triggered and react accordingly.
        let (shutdown_watch_tx, shutdown_watch_rx) = watch::channel(());

        dal::init()?;

        let encryption_key =
            Self::load_encryption_key(config.cyclone_encryption_key_path()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let faktory = Self::create_faktory_client(config.faktory());

        let graceful_shutdown_rx =
            prepare_graceful_shutdown(external_shutdown_rx, shutdown_watch_tx)?;

        Ok(Server {
            encryption_key,
            nats,
            pg_pool,
            veritech,
            faktory,
            shutdown_watch_rx,
            external_shutdown_tx,
            graceful_shutdown_rx,
        })
    }

    /// Gets a [`ShutdownHandle`] that can externally or on demand trigger the server's shutdown
    /// process.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            shutdown_tx: self.external_shutdown_tx.clone(),
        }
    }

    pub async fn run(self) -> Result<()> {
        const NUM_TASKS: usize = 10;
        let mut handles = Vec::with_capacity(NUM_TASKS);

        let (processor_alive_marker_tx, mut processor_alive_marker_rx) = mpsc::channel(1);

        for _ in 0..NUM_TASKS {
            handles.push(tokio::task::spawn(start_job_executor(
                self.faktory.clone(),
                self.pg_pool.clone(),
                self.nats.clone(),
                self.veritech.clone(),
                self.encryption_key.clone(),
                self.shutdown_watch_rx.clone(),
                processor_alive_marker_tx.clone(),
            )));
        }
        // Drop the remaining extra clone, ensuring that the only senders are in job executors
        drop(processor_alive_marker_tx);

        let _ = futures::future::join_all(handles).await;

        // Blocks until all FaktoryProcessors are gone so we don't skip jobs that are still being
        // sent to faktory_async
        info!("waiting for all job processors to finish pushing jobs");
        let _ = processor_alive_marker_rx.recv().await;

        info!("closing faktory-async client connection");
        if let Err(err) = self.faktory.close().await {
            warn!(error = ?err, "failed to cleanly close fakory-async client connection");
        }

        let _ = self.graceful_shutdown_rx.await;
        info!("received and processed graceful shutdown, terminating server instance");

        Ok(())
    }

    #[instrument(name = "pinga.init.load_encryption_key", skip_all)]
    async fn load_encryption_key(path: impl AsRef<Path>) -> Result<Arc<EncryptionKey>> {
        Ok(Arc::new(EncryptionKey::load(path).await?))
    }

    #[instrument(name = "pinga.init.connect_to_nats", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "pinga.init.create_pg_pool", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "pinga.init.create_veritech_client", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "pinga.init.create_faktory_client", skip_all)]
    fn create_faktory_client(faktory_config: &FaktoryConfig) -> FaktoryClient {
        let config = si_data_faktory::Config::from_uri(
            &faktory_config.url,
            Some("pinga".to_string()),
            Some(Uuid::new_v4().to_string()),
        );
        let client = FaktoryClient::new(config, 256);
        debug!("successfully spawned faktory client connection");
        client
    }
}

pub struct ShutdownHandle {
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl ShutdownHandle {
    pub async fn shutdown(self) {
        if let Err(err) = self.shutdown_tx.send(ShutdownSource::Handle).await {
            warn!(error = ?err, "shutdown tx returned error, receiver is likely already closed");
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {
    Handle,
}

impl Default for ShutdownSource {
    fn default() -> Self {
        Self::Handle
    }
}

/// Start the faktory job executor
async fn start_job_executor(
    client: FaktoryClient,
    pg: PgPool,
    nats: NatsClient,
    veritech: VeritechClient,
    encryption_key: Arc<EncryptionKey>,
    mut shutdown_watch_rx: watch::Receiver<()>,
    processor_alive_marker_tx: mpsc::Sender<()>,
) {
    let job_processor = Box::new(FaktoryProcessor::new(
        client.clone(),
        processor_alive_marker_tx,
    )) as Box<dyn JobQueueProcessor + Send + Sync>;
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech.clone(),
        encryption_key,
    );
    let ctx_builder = DalContext::builder(services_context);

    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;

        match client.last_beat().await {
            Ok(BeatState::Ok) => {
                let job = tokio::select! {
                    job = client.fetch(vec!["default".into()]) => match job {
                        Ok(Some(job)) => job,
                        Ok(None) => continue,
                        Err(err) => {
                            error!("Unable to fetch from faktory: {err}");
                            continue;
                        }
                    },
                    _ = shutdown_watch_rx.changed() => {
                        info!("Worker task received shutdown notification: stopping");
                        break;
                    }
                };

                let jid = job.id().to_owned();
                match execute_job_fallible(job, ctx_builder.clone()).await {
                    Ok(()) => match client.ack(jid).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!("Ack failed: {err}");
                            continue;
                        }
                    },
                    Err(err) => {
                        error!("Job execution failed: {err}");
                        // TODO: pass backtrace here
                        match client
                            .fail(FailConfig::new(
                                jid,
                                format!("{err:?}"),
                                err.to_string(),
                                None,
                            ))
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Fail failed: {err}");
                                continue;
                            }
                        }
                    }
                }
            }
            Ok(BeatState::Quiet) => {
                // Getting a "quiet" state from the faktory server means that
                // someone has gone to the faktory UI and requested that this
                // particular worker finish what it's doing, and gracefully
                // shut down.
                info!("Gracefully shutting down from Faktory request.");
                break;
            }
            Ok(BeatState::Terminate) => {
                warn!("Faktory asked us to terminate");
                break;
            }
            Err(err) => {
                error!("Internal error in faktory-async, bailing out: {err}");
                break;
            }
        }
    }
}

macro_rules! job_match {
    ($job:ident, $( $job_struct:ident ),* $(,)* ) => {
        match $job.kind() {
            $(
                stringify!($job_struct) => {
                    Ok(
                        Box::new($job_struct::try_from($job)?) as Box<dyn ::dal::job::consumer::JobConsumer + Send + Sync>
                    )
                }
            )*
            kind => Err(JobError::UnknownJobKind(kind.to_owned())),
        }
    };
}

async fn execute_job_fallible(
    job: si_data_faktory::Job,
    ctx_builder: DalContextBuilder,
) -> ::std::result::Result<(), JobError> {
    info!("Processing {job:?}");

    let job = match job_match!(
        job,
        Confirmation,
        Confirmations,
        Qualification,
        Qualifications,
        CodeGeneration,
        DependentValuesUpdate,
        WorkflowRun,
        FixesJob,
    ) {
        Ok(job) => job,
        Err(err) => return Err(err),
    };

    let outer_ctx_builder = ctx_builder.clone();
    let access_builder = job.access_builder();
    let visibility = job.visibility();
    let job_type_name = job.type_name();

    let result = panic_wrapper(
        format!("Job execution: {job_type_name}"),
        job.run_job(outer_ctx_builder),
    )
    .await;

    if let Err(err) = result {
        error!("Job execution failed: {err}");

        let err_message = err.to_string();
        panic_wrapper(format!("Job failure reporting: {job_type_name}"), async {
            let ctx = ctx_builder.build(access_builder.build(visibility)).await?;

            JobFailure::new(&ctx, job_type_name.clone(), err_message).await?;

            ctx.commit().await?;
            ::std::result::Result::<(), JobFailureError>::Ok(())
        })
        .await?;

        return Err(err);
    }
    Ok(())
}

async fn panic_wrapper<
    E: std::error::Error + Send + Sync + 'static,
    F: Future<Output = ::std::result::Result<(), E>>,
>(
    label: String,
    future: F,
) -> ::std::result::Result<(), JobError> {
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(JobError::ExecutionFailed(label, Box::new(err))),
        Err(any) => {
            // Note: Technically panics can be of any form, but most should be &str or String
            match any.downcast::<String>() {
                Ok(msg) => Err(JobError::Panic(label, *msg)),
                Err(any) => match any.downcast::<&str>() {
                    Ok(msg) => Err(JobError::Panic(label, msg.to_string())),
                    Err(any) => {
                        let id = any.type_id();
                        error!("{label}: Panic message downcast failed of {id:?}",);
                        Err(JobError::UnknownPanic(label, id))
                    }
                },
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum JobError {
    #[error("unknown job kind {0}")]
    UnknownJobKind(String),
    #[error("execution failed for {0}: {1}")]
    ExecutionFailed(String, Box<dyn std::error::Error + Send + Sync>),
    #[error("{0} panicked: {1}")]
    Panic(String, String),
    #[error("{0} panicked with an unknown payload of {1:?}")]
    UnknownPanic(String, TypeId),

    #[error(transparent)]
    FailureReporting(#[from] JobFailureError),
    #[error(transparent)]
    Pg(#[from] PgPoolError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Faktory(#[from] si_data_faktory::Error),
    #[error(transparent)]
    JobConsumer(#[from] JobConsumerError),
    #[error(transparent)]
    JoinError(#[from] JoinError),
}

fn prepare_graceful_shutdown(
    mut external_shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_watch_tx: watch::Sender<()>,
) -> Result<oneshot::Receiver<()>> {
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
