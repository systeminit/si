use std::collections::HashMap;
use std::{io, path::Path, sync::Arc};

use dal::{
    job::consumer::JobConsumerError, DalContext, InitializationError, JobFailureError,
    JobQueueProcessor, NatsProcessor, ServicesContext, TransactionsError,
};
use nats_subscriber::SubscriberError;
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use si_rabbitmq::{
    Consumer, ConsumerHandle, ConsumerOffsetSpecification, Environment, Producer, RabbitError,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    signal::unix,
    sync::{
        mpsc::{self},
        oneshot, watch,
    },
};
use ulid::Ulid;
use veritech_client::{Client as VeritechClient, EncryptionKey, EncryptionKeyError};

use crate::GOBBLER_STREAM_PREFIX;
use crate::{Config, GOBBLER_MANAGEMENT_STREAM};
use crate::{ManagementMessage, ManagementMessageAction};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
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

/// The [`Server`] for managing gobbler tasks.
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
    #[instrument(name = "gobbler.init.from_config", skip_all)]
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
    #[instrument(name = "gobbler.init.from_services", skip_all)]
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
        consume_stream_task(
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

    #[instrument(name = "gobbler.init.load_encryption_key", skip_all)]
    async fn load_encryption_key(path: impl AsRef<Path>) -> ServerResult<Arc<EncryptionKey>> {
        Ok(Arc::new(EncryptionKey::load(path).await?))
    }

    #[instrument(name = "gobbler.init.connect_to_nats", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "gobbler.init.create_pg_pool", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> ServerResult<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "gobbler.init.create_veritech_client", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "gobbler.init.create_job_processor", skip_all)]
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

#[allow(clippy::too_many_arguments)]
async fn consume_stream_task(
    recreate_management_stream: bool,
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    encryption_key: Arc<veritech_client::EncryptionKey>,
    shutdown_watch_rx: watch::Receiver<()>,
) {
    if let Err(err) = consume_stream(
        recreate_management_stream,
        pg_pool,
        nats,
        veritech,
        job_processor,
        encryption_key,
        shutdown_watch_rx,
    )
    .await
    {
        info!(error = ?err, "consuming stream failed");
    }
}

#[allow(clippy::too_many_arguments)]
async fn consume_stream(
    recreate_management_stream: bool,
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    encryption_key: Arc<veritech_client::EncryptionKey>,
    mut shutdown_watch_rx: watch::Receiver<()>,
) -> ServerResult<()> {
    let services_context = ServicesContext::new(
        pg_pool,
        nats.clone(),
        job_processor,
        veritech.clone(),
        encryption_key,
        None,
        None,
    );
    let _ctx_builder = DalContext::builder(services_context, false);

    // Meta: we can only have one gobbler instance right now due to https://github.com/rabbitmq/rabbitmq-stream-rust-client/issues/130
    //
    // 1) subscribe to "next" for changeset close/create events --> stream for ChangeSetClose or ChangeSetOpen
    //    --> "gobbler-management"
    // 2) query db for all named, open changesets
    // 3) start a subscription for each result for step 2
    //    --> "gobbler-<change-set-id>"
    //    1:N --> "gobbler-<change-set-id>-reply-<requester>-<ulid>"
    //      (e.g. "gobbler-<change-set-id>-reply-sdf-<ulid>")
    //             note: requester deletes stream upon reply
    //
    // NOTE: QUERY DB FOR OFFSET NUMBER OR GO TO FIRST SPECIFICATION

    // Prepare the environment and management stream.
    let environment = Environment::new().await?;
    if recreate_management_stream {
        environment.delete_stream(GOBBLER_MANAGEMENT_STREAM).await?;
    }
    environment.create_stream(GOBBLER_MANAGEMENT_STREAM).await?;

    let mut management_consumer = Consumer::new(
        &environment,
        GOBBLER_MANAGEMENT_STREAM,
        ConsumerOffsetSpecification::Next,
    )
    .await?;
    let management_handle = management_consumer.handle();
    let mut gobbler_handles: HashMap<Ulid, (String, ConsumerHandle)> = HashMap::new();

    while let Some(management_delivery) = management_consumer.next().await? {
        let contents = management_delivery
            .message_contents
            .ok_or(ServerError::MissingManagementMessageContents)?;
        let reply_to = management_delivery
            .reply_to
            .ok_or(ServerError::MissingManagementMessageReplyTo)?;
        let mm: ManagementMessage = serde_json::from_value(contents)?;

        match mm.action {
            ManagementMessageAction::Close => match gobbler_handles.remove(&mm.change_set_id) {
                Some((stream, handle)) => {
                    if let Err(e) = handle.close().await {
                        error!("{e}");
                    }
                    if let Err(e) = environment.delete_stream(stream).await {
                        error!("{e}");
                    }
                }
                None => debug!(
                    "did not find handle for change set id: {}",
                    mm.change_set_id
                ),
            },
            ManagementMessageAction::Open => {
                let new_stream = format!("{GOBBLER_STREAM_PREFIX}-{}", mm.change_set_id);
                let stream_already_exists = environment.create_stream(&new_stream).await?;

                // Only create the new stream if it does not already exist.
                if !stream_already_exists {
                    let consumer =
                        Consumer::new(&environment, &new_stream, ConsumerOffsetSpecification::Next)
                            .await?;
                    let handle = consumer.handle();
                    gobbler_handles.insert(mm.change_set_id, (new_stream.clone(), handle));

                    tokio::spawn(gobbler_loop_infallible_wrapper(consumer));
                }

                // Return the requested stream and then close the producer.
                let mut producer = Producer::for_reply(&environment, &new_stream, reply_to).await?;
                producer.send_single(new_stream, None).await?;
                producer.close().await?;
            }
        }
    }

    for (_, (stream, handle)) in gobbler_handles.drain() {
        if let Err(e) = handle.close().await {
            error!("{e}");
        }
        if let Err(e) = environment.delete_stream(stream).await {
            error!("{e}")
        }
    }
    if let Err(e) = management_handle.close().await {
        error!("{e}");
    }
    Ok(())
}

async fn gobbler_loop_infallible_wrapper(consumer: Consumer) {
    if let Err(e) = gobbler_loop(consumer).await {
        dbg!(e);
    }
}

async fn gobbler_loop(mut consumer: Consumer) -> ServerResult<()> {
    // Create an environment for reply streams.
    let environment = Environment::new().await?;
    while let Some(delivery) = consumer.next().await? {
        if let Some(reply_to) = delivery.reply_to {
            let mut producer =
                Producer::for_reply(&environment, consumer.stream(), reply_to).await?;

            // -----------------------------------------
            // TODO(nick): this is where the fun begins.
            //   1) succeed everywhere
            //   2) store offset with changeset
            //   3) update requester stream w/out waiting for reply
            // -----------------------------------------

            // TODO(nick): for now, just send back the message. Unwrapping is fine because we know
            // that it must have content.
            producer
                .send_single(delivery.message_contents.unwrap(), None)
                .await?;
            producer.close().await?;
        }
    }
    Ok(())
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
