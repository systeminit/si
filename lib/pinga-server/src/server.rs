use std::{
    future::IntoFuture,
    io,
    sync::atomic::{self, AtomicUsize},
    sync::Arc,
};

use dal::job::definition::compute_validation::ComputeValidation;
use dal::{
    job::{
        consumer::{JobConsumer, JobConsumerError, JobInfo},
        definition::{ActionJob, DependentValuesUpdate, DeprecatedActionsJob, RefreshJob},
        producer::BlockingJobError,
    },
    DalContext, DalContextBuilder, InitializationError, JobFailure, JobFailureError,
    JobQueueProcessor, NatsProcessor, ServicesContext, TransactionsError,
};
use futures::{FutureExt, Stream, StreamExt};
use nats_subscriber::{Request, SubscriberError};
use si_crypto::{
    CryptoConfig, SymmetricCryptoError, SymmetricCryptoService, SymmetricCryptoServiceConfig,
};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use si_layer_cache::{error::LayerDbError, LayerDb};
use stream_cancel::StreamExt as StreamCancelStreamExt;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot, watch,
    },
    task,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use veritech_client::{Client as VeritechClient, CycloneEncryptionKey, CycloneEncryptionKeyError};

use crate::{nats_jobs_subject, Config, NATS_JOBS_DEFAULT_QUEUE};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("error when loading cyclone encryption key: {0}")]
    EncryptionKey(#[from] CycloneEncryptionKeyError),
    #[error(transparent)]
    Initialization(#[from] InitializationError),
    #[error(transparent)]
    JobConsumer(#[from] JobConsumerError),
    #[error(transparent)]
    JobFailure(#[from] Box<JobFailureError>),
    #[error("layer cache error: {0}")]
    LayerCache(#[from] LayerDbError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error(transparent)]
    SymmetricCryptoService(#[from] SymmetricCryptoError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
    #[error("unable to connect to database: {0}")]
    UnableToConnectToDatabase(Box<PgPoolError>),
    #[error("unknown job kind {0}")]
    UnknownJobKind(String),
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

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server {
    concurrency_limit: usize,
    services_context: ServicesContext,
    /// An internal shutdown watch receiver handle which can be provided to internal tasks which
    /// want to be notified when a shutdown event is in progress.
    shutdown_watch_rx: watch::Receiver<()>,
    /// An external shutdown sender handle which can be handed out to external callers who wish to
    /// trigger a server shutdown at will.
    external_shutdown_tx: mpsc::Sender<ShutdownSource>,
    /// An internal graceful shutdown receiever handle which the server's main thread uses to stop
    /// accepting work when a shutdown event is in progress.
    graceful_shutdown_rx: oneshot::Receiver<()>,
    metadata: Arc<ServerMetadata>,
}

impl Server {
    #[instrument(name = "pinga.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        token: CancellationToken,
        tracker: TaskTracker,
    ) -> Result<Self> {
        dal::init()?;

        let encryption_key = Self::load_encryption_key(config.crypto().clone()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone());
        let symmetric_crypto_service =
            Self::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

        let (layer_db, layer_db_graceful_shutdown) =
            LayerDb::from_config(config.layer_db_config().clone(), token).await?;
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
        );

        Self::from_services(
            config.instance_id().to_string(),
            config.concurrency(),
            services_context,
        )
    }

    #[instrument(name = "pinga.init.from_services", level = "info", skip_all)]
    pub fn from_services(
        instance_id: impl Into<String>,
        concurrency_limit: usize,
        services_context: ServicesContext,
    ) -> Result<Self> {
        // An mpsc channel which can be used to externally shut down the server.
        let (external_shutdown_tx, external_shutdown_rx) = mpsc::channel(4);
        // A watch channel used to notify internal parts of the server that a shutdown event is in
        // progress. The value passed along is irrelevant--we only care that the event was
        // triggered and react accordingly.
        let (shutdown_watch_tx, shutdown_watch_rx) = watch::channel(());

        dal::init()?;

        let metadata = ServerMetadata {
            job_instance: instance_id.into(),
            job_invoked_provider: "si",
        };

        let graceful_shutdown_rx =
            prepare_graceful_shutdown(external_shutdown_rx, shutdown_watch_tx)?;

        Ok(Server {
            concurrency_limit,
            services_context,
            shutdown_watch_rx,
            external_shutdown_tx,
            graceful_shutdown_rx,
            metadata: Arc::new(metadata),
        })
    }

    pub async fn run(self) -> Result<()> {
        // First, check if we can communicate with the database. If we cannot, we need to explode.
        self.services_context
            .pg_pool()
            .test_connection()
            .await
            .map_err(|e| ServerError::UnableToConnectToDatabase(Box::new(e)))?;

        let (tx, rx) = mpsc::unbounded_channel();

        // Span a task to receive and process jobs from the unbounded channel
        drop(task::spawn(process_job_requests_task(
            rx,
            self.concurrency_limit,
        )));

        // Run "the main loop" which pulls message from a subscription off NATS and forwards each
        // request to an unbounded channel
        receive_job_requests_task(
            tx,
            self.metadata,
            self.services_context,
            self.shutdown_watch_rx,
        )
        .await;

        let _ = self.graceful_shutdown_rx.await;
        info!("received and processed graceful shutdown, terminating server instance");

        Ok(())
    }

    /// Gets a [`ShutdownHandle`](PingaShutdownHandle) that can externally or on demand trigger the server's shutdown
    /// process.
    pub fn shutdown_handle(&self) -> PingaShutdownHandle {
        PingaShutdownHandle {
            shutdown_tx: self.external_shutdown_tx.clone(),
        }
    }

    #[instrument(name = "pinga.init.load_encryption_key", level = "info", skip_all)]
    async fn load_encryption_key(crypto_config: CryptoConfig) -> Result<Arc<CycloneEncryptionKey>> {
        Ok(Arc::new(
            CycloneEncryptionKey::from_config(crypto_config).await?,
        ))
    }

    #[instrument(name = "pinga.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "pinga.init.create_pg_pool", level = "info", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
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
    ) -> Result<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub struct ServerMetadata {
    job_instance: String,
    job_invoked_provider: &'static str,
}

pub struct PingaShutdownHandle {
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl PingaShutdownHandle {
    pub async fn shutdown(self) {
        if let Err(err) = self.shutdown_tx.send(ShutdownSource::Handle).await {
            warn!(error = ?err, "shutdown tx returned error, receiver is likely already closed");
        }
    }
}

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {
    Handle,
}

impl Default for ShutdownSource {
    fn default() -> Self {
        Self::Handle
    }
}

pub struct JobItem {
    metadata: Arc<ServerMetadata>,
    ctx_builder: DalContextBuilder,
    request: Result<Request<JobInfo>>,
}

pub struct Subscriber;

impl Subscriber {
    pub async fn jobs(
        metadata: Arc<ServerMetadata>,
        services_context: ServicesContext,
    ) -> Result<impl Stream<Item = JobItem>> {
        let nats = services_context.nats_conn().clone();

        let subject = nats_jobs_subject(nats.metadata().subject_prefix());
        debug!(
            messaging.destination.name = subject.as_str(),
            "subscribing for job requests"
        );

        // Make non blocking context here, and update it for each job
        // Since the any blocking job should block on its child jobs
        let ctx_builder = DalContext::builder(services_context, false);

        Ok(nats_subscriber::Subscriber::create(subject)
            .queue_name(NATS_JOBS_DEFAULT_QUEUE)
            .start(&nats)
            .await?
            .map(move |request| JobItem {
                metadata: metadata.clone(),
                ctx_builder: ctx_builder.clone(),
                request: request.map_err(Into::into),
            }))
    }
}

async fn receive_job_requests_task(
    tx: UnboundedSender<JobItem>,
    metadata: Arc<ServerMetadata>,
    services_context: ServicesContext,
    shutdown_watch_rx: watch::Receiver<()>,
) {
    if let Err(err) = receive_job_requests(tx, metadata, services_context, shutdown_watch_rx).await
    {
        warn!(error = ?err, "processing job requests failed");
    }
}

async fn receive_job_requests(
    tx: UnboundedSender<JobItem>,
    metadata: Arc<ServerMetadata>,
    services_context: ServicesContext,
    mut shutdown_watch_rx: watch::Receiver<()>,
) -> Result<()> {
    let mut requests = Subscriber::jobs(metadata, services_context)
        .await?
        .take_until_if(Box::pin(shutdown_watch_rx.changed().map(|_| true)));

    // Forward each request off the stream to a consuming task via an *unbounded* channel so we
    // buffer requests until we run out of memory. Have fun!
    while let Some(job) = requests.next().await {
        if let Err(_job) = tx.send(job) {
            error!("process_job_requests rx has already closed");
        }
    }

    Ok(())
}

static CONCURRENT_TASKS: AtomicUsize = AtomicUsize::new(0);

#[instrument(level = "info", skip(rx))]
async fn process_job_requests_task(rx: UnboundedReceiver<JobItem>, concurrency_limit: usize) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(concurrency_limit, |job| async move {
            let concurrency_count = CONCURRENT_TASKS.fetch_add(1, atomic::Ordering::Relaxed) + 1;

            let span = Span::current();
            span.record("concurrency.count", concurrency_count);

            // Got the next message from the subscriber
            trace!("pulled request into an available concurrent task");

            match job.request {
                Ok(request) => {
                    // Spawn a task and process the request
                    let join_handle = task::spawn(execute_job_task(
                        job.metadata,
                        job.ctx_builder,
                        request,
                        concurrency_count,
                        concurrency_limit,
                    ));
                    if let Err(err) = join_handle.await {
                        // NOTE(fnichol): This likely happens when there is contention or
                        // an error in the Tokio runtime so we will be loud and log an
                        // error under the assumptions that 1) this event rarely
                        // happens and 2) the task code did not contribute to trigger
                        // the `JoinError`.
                        error!(
                            error = ?err,
                            "execute-job-task failed to execute to completion"
                        );
                    };
                }
                Err(err) => {
                    warn!(error = ?err, "next job request had an error, job will not be executed");
                }
            }

            let concurrency_count = CONCURRENT_TASKS.fetch_sub(1, atomic::Ordering::Relaxed) - 1;

            let span = Span::current();
            span.record("concurrency.count", concurrency_count);
        })
        .await;
}

#[instrument(
    name = "execute_job_task",
    parent = &request.process_span,
    level = "info",
    skip_all,
    fields(
        job.blocking = request.payload.blocking,
        job.id = request.payload.id,
        job.instance = metadata.job_instance,
        job.invoked_args = Empty,
        job.invoked_name = request.payload.kind,
        job.invoked_provider = metadata.job_invoked_provider,
        job.trigger = "pubsub",
        job.visibility = ?request.payload.visibility,
        concurrency.count = concurrency_count,
        concurrency.limit = concurrency_limit,
        concurrency.at_capacity = concurrency_limit == concurrency_count,
        messaging.destination = Empty,
        messaging.destination_kind = "topic",
        messaging.operation = "process",
        otel.kind = SpanKind::Consumer.as_str(),
        otel.name = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
)]
async fn execute_job_task(
    metadata: Arc<ServerMetadata>,
    ctx_builder: DalContextBuilder,
    request: Request<JobInfo>,
    concurrency_count: usize,
    concurrency_limit: usize,
) {
    let span = Span::current();
    let id = request.payload.id.clone();

    let arg_str = serde_json::to_string(&request.payload.arg)
        .unwrap_or_else(|_| "arg failed to serialize".to_string());

    let messaging_destination = request.subject;

    span.record("job.invoked_arg", arg_str);
    span.record("messaging.destination", messaging_destination.as_str());
    span.record(
        "otel.name",
        format!("{} process", &messaging_destination).as_str(),
    );

    let maybe_reply_channel = request.reply.clone();
    let reply_message = match execute_job(ctx_builder.clone(), request.payload).await {
        Ok(_) => {
            span.record_ok();
            Ok(())
        }
        Err(err) => {
            error!(
                error = ?err,
                job.invocation_id = %id,
                job.instance = &metadata.job_instance,
                "job execution failed"
            );
            let new_err = Err(BlockingJobError::JobExecution(err.to_string()));
            span.record_err(err);

            new_err
        }
    };

    if let Some(reply_channel) = maybe_reply_channel {
        if let Ok(message) = serde_json::to_vec(&reply_message) {
            if let Err(err) = ctx_builder
                .nats_conn()
                .publish(reply_channel, message.into())
                .await
            {
                error!(error = ?err, "Unable to notify spawning job of blocking job completion");
            };
        }
    }
}

async fn execute_job(mut ctx_builder: DalContextBuilder, job_info: JobInfo) -> Result<()> {
    if job_info.blocking {
        ctx_builder.set_blocking();
    }

    let job = match job_info.kind.as_str() {
        stringify!(DependentValuesUpdate) => {
            Box::new(DependentValuesUpdate::try_from(job_info.clone())?)
                as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(ActionJob) => {
            Box::new(ActionJob::try_from(job_info.clone())?) as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(DeprecatedActionsJob) => {
            Box::new(DeprecatedActionsJob::try_from(job_info.clone())?)
                as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(RefreshJob) => {
            Box::new(RefreshJob::try_from(job_info.clone())?) as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(ComputeValidation) => Box::new(ComputeValidation::try_from(job_info.clone())?)
            as Box<dyn JobConsumer + Send + Sync>,
        //     stringify!(RefreshJob) => Box::new(RefreshJob::try_from(job_info.clone())?)
        //         as Box<dyn JobConsumer + Send + Sync>,
        kind => return Err(ServerError::UnknownJobKind(kind.to_owned())),
    };

    info!("Processing job");

    if let Err(err) = job.run_job(ctx_builder.clone()).await {
        // The missing part is this, should we execute subsequent jobs if the one they depend on fail or not?
        record_job_failure(ctx_builder, job, err).await?;
    }

    info!("Finished processing job");

    Ok(())
}

#[allow(dead_code)]
async fn record_job_failure(
    ctx_builder: DalContextBuilder,
    job: Box<dyn JobConsumer + Send + Sync>,
    err: JobConsumerError,
) -> Result<()> {
    warn!(error = ?err, "job execution failed, recording a job failure to the database");

    let access_builder = job.access_builder();
    let visibility = job.visibility();
    let ctx = ctx_builder.build(access_builder.build(visibility)).await?;

    JobFailure::new(&ctx, job.type_name(), err.to_string()).await?;

    ctx.commit().await?;

    Err(err.into())
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
