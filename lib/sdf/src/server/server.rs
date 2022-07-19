use std::{
    any::TypeId,
    io,
    net::SocketAddr,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::server::config::{CycloneKeyPair, JwtSecretKey};
use axum::routing::IntoMakeService;
use axum::Router;
use dal::{
    cyclone_key_pair::CycloneKeyPairError,
    job::{
        consumer::{JobConsumer, JobConsumerError},
        definition::{
            component_post_processing::ComponentPostProcessing,
            dependent_values_update::DependentValuesUpdate,
        },
        processor::JobQueueProcessor,
    },
    jwt_key::{install_new_jwt_key, jwt_key_exists},
    migrate, migrate_builtins, DalContext, DalContextBuilder, JobFailure, JobFailureError,
    ResourceScheduler, ServicesContext, TransactionsError,
};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use si_data::{
    NatsClient, NatsConfig, NatsError, PgError, PgPool, PgPoolConfig, PgPoolError, SensitiveString,
};
use telemetry::{prelude::*, TelemetryClient};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    runtime::Runtime,
    signal::unix,
    sync::{broadcast, mpsc, oneshot},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("error initializing the server")]
    Init,
    #[error("jwt secret key error")]
    JwtSecretKey(#[from] dal::jwt_key::JwtKeyError),
    #[error(transparent)]
    Model(#[from] dal::ModelError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Uds(#[from] UdsIncomingStreamError),
    #[error("cyclone public key error: {0}")]
    CyclonePublicKeyErr(#[from] CycloneKeyPairError),
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
    #[error("cyclone public key already set")]
    CyclonePublicKeyAlreadySet,
    #[error("error when loading encryption key: {0}")]
    EncryptionKey(#[from] veritech::EncryptionKeyError),
}

pub type Result<T, E = ServerError> = std::result::Result<T, E>;

pub struct Server<I, S> {
    config: Config,
    inner: axum::Server<I, IntoMakeService<Router>>,
    socket: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server<(), ()> {
    #[allow(clippy::too_many_arguments)]
    pub fn http(
        config: Config,
        telemetry: telemetry::Client,
        pg_pool: PgPool,
        nats: NatsClient,
        job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
        veritech: veritech::Client,
        encryption_key: veritech::EncryptionKey,
        jwt_secret_key: JwtSecretKey,
    ) -> Result<Server<AddrIncoming, SocketAddr>> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx) = build_service(
                    telemetry,
                    pg_pool,
                    nats,
                    job_processor,
                    veritech,
                    encryption_key,
                    jwt_secret_key,
                    config.signup_secret().clone(),
                )?;

                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                let inner = axum::Server::bind(socket_addr).serve(service.into_make_service());
                let socket = inner.local_addr();

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong @ IncomingStream::UnixDomainSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn uds(
        config: Config,
        telemetry: telemetry::Client,
        pg_pool: PgPool,
        nats: NatsClient,
        job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
        veritech: veritech::Client,
        encryption_key: veritech::EncryptionKey,
        jwt_secret_key: JwtSecretKey,
    ) -> Result<Server<UdsIncomingStream, PathBuf>> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx) = build_service(
                    telemetry,
                    pg_pool,
                    nats,
                    job_processor,
                    veritech,
                    encryption_key,
                    jwt_secret_key,
                    config.signup_secret().clone(),
                )?;

                info!("binding to Unix domain socket; path={}", path.display());
                let inner = axum::Server::builder(UdsIncomingStream::create(path).await?)
                    .serve(service.into_make_service());
                let socket = path.clone();

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong @ IncomingStream::HTTPSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    pub fn init() -> Result<()> {
        sodiumoxide::init().map_err(|_| ServerError::Init)
    }

    #[instrument(name = "sdf.init.generate_jwt_secret_key", skip_all)]
    pub async fn generate_jwt_secret_key(path: impl AsRef<Path>) -> Result<JwtSecretKey> {
        Ok(JwtSecretKey::create(path).await?)
    }

    #[instrument(name = "sdf.init.generate_cyclone_key_pair", skip_all)]
    pub async fn generate_cyclone_key_pair(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<()> {
        CycloneKeyPair::create(secret_key_path, public_key_path)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "sdf.init.load_jwt_secret_key", skip_all)]
    pub async fn load_jwt_secret_key(path: impl AsRef<Path>) -> Result<JwtSecretKey> {
        Ok(JwtSecretKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.load_encryption_key", skip_all)]
    pub async fn load_encryption_key(path: impl AsRef<Path>) -> Result<veritech::EncryptionKey> {
        Ok(veritech::EncryptionKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.migrate_database", skip_all)]
    pub async fn migrate_database(
        pg: &PgPool,
        nats: &NatsClient,
        job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
        jwt_secret_key: &JwtSecretKey,
        veritech: veritech::Client,
        encryption_key: &veritech::EncryptionKey,
    ) -> Result<()> {
        migrate(pg).await?;
        migrate_builtins(pg, nats, job_processor, veritech, encryption_key).await?;

        let mut conn = pg.get().await?;
        let txn = conn.transaction().await?;
        if !jwt_key_exists(&txn).await? {
            debug!("no jwt key found, generating new keypair");
            install_new_jwt_key(&txn, jwt_secret_key).await?;
        }
        // TODO: manually process job queue

        txn.commit().await?;

        Ok(())
    }

    /// Start the basic resource sync scheduler
    pub async fn start_resource_sync_scheduler(
        pg: PgPool,
        nats: NatsClient,
        job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
        veritech: veritech::Client,
        encryption_key: veritech::EncryptionKey,
    ) {
        let services_context =
            ServicesContext::new(pg, nats, job_processor, veritech, Arc::new(encryption_key));
        let scheduler = ResourceScheduler::new(services_context);
        tokio::spawn(scheduler.start());
    }

    /// Start the faktory job executor
    pub async fn start_job_executor(
        pg: PgPool,
        nats: NatsClient,
        job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
        job_executor_config: String,
        veritech: veritech::Client,
        encryption_key: veritech::EncryptionKey,
        runtime: Arc<Runtime>,
    ) {
        let services_context =
            ServicesContext::new(pg, nats, job_processor, veritech, Arc::new(encryption_key));
        let ctx_builder = Arc::new(DalContext::builder(services_context));

        loop {
            let mut c = faktory::ConsumerBuilder::default();

            let ctx_builder1 = ctx_builder.clone();
            let runtime1 = runtime.clone();
            // TODO: we should have a way to report failures without access_builder + visibility, that is only readable to us, as it implies in a dal's implementation problem
            c.register("ComponentPostProcessing", move |job| -> io::Result<()> {
                info!("Got a ComponentPostProcessing job: {}", job.id());
                faktory_job_wrapper(
                    ctx_builder1.clone(),
                    Box::new(ComponentPostProcessing::try_from(job)?)
                        as Box<dyn JobConsumer + Send + Sync>,
                    runtime1.clone(),
                )
            });

            let ctx_builder2 = ctx_builder.clone();
            let runtime2 = runtime.clone();
            c.register("DependentValuesUpdate", move |job| -> io::Result<()> {
                info!("Got a DependentValuesUpdate job: {}", job.id());
                faktory_job_wrapper(
                    ctx_builder2.clone(),
                    Box::new(DependentValuesUpdate::try_from(job)?)
                        as Box<dyn JobConsumer + Send + Sync>,
                    runtime2.clone(),
                )
            });

            let mut c = match c.connect(Some(&job_executor_config)) {
                Ok(c) => {
                    info!("Conected to faktory at: {job_executor_config}");

                    c
                }
                Err(err) => {
                    error!("Unable to connect to faktory at {job_executor_config}: {err}");
                    std::thread::sleep(Duration::from_millis(5000));
                    continue;
                }
            };

            if let Err(e) = c.run(&["default"]) {
                error!("worker failed: {}", e);
            }
        }
    }

    #[instrument(name = "sdf.init.create_pg_pool", skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "sdf.init.connect_to_nats", skip_all)]
    pub async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    pub fn create_veritech_client(nats: NatsClient) -> veritech::Client {
        veritech::Client::new(nats)
    }
}

impl<I, IO, IE, S> Server<I, S>
where
    I: Accept<Conn = IO, Error = IE>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn run(self) -> Result<()> {
        let shutdown_rx = self.shutdown_rx;

        self.inner
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .map_err(Into::into)
    }

    /// Gets a reference to the server's config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Gets a reference to the server's locally bound socket.
    pub fn local_socket(&self) -> &S {
        &self.socket
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_service(
    telemetry: impl TelemetryClient,
    pg_pool: PgPool,
    nats: NatsClient,
    job_processor: Arc<Box<dyn JobQueueProcessor + Send + Sync>>,
    veritech: veritech::Client,
    encryption_key: veritech::EncryptionKey,
    jwt_secret_key: JwtSecretKey,
    signup_secret: SensitiveString,
) -> Result<(Router, oneshot::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(4);
    // Note the channel parameter corresponds to the number of channels that may be maintained when
    // the sender is guaranteeing delivery. While this number may end of being related to the
    // number of active WebSocket sessions, it's not necessarily the same number.
    let (shutdown_broadcast_tx, _) = broadcast::channel(512);

    let routes = routes(
        telemetry,
        pg_pool,
        nats,
        job_processor,
        veritech,
        encryption_key,
        jwt_secret_key,
        signup_secret,
        shutdown_tx,
        shutdown_broadcast_tx.clone(),
    )
    // TODO(fnichol): customize http tracing further, using:
    // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
    .layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    let graceful_shutdown_rx = prepare_graceful_shutdown(shutdown_rx, shutdown_broadcast_tx)?;

    Ok((routes, graceful_shutdown_rx))
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(
            tx: oneshot::Sender<()>,
            shutdown_broadcast_tx: broadcast::Sender<()>,
        ) {
            // Send graceful shutdown to axum server which stops it from accepting requests
            if tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
            // Send shutdown to all long running sessions (notably, WebSocket sessions), so they
            // can cleanly terminate
            if shutdown_broadcast_tx.send(()).is_err() {
                error!("all broadcast shutdown receivers have already been dropped");
            }
        }

        tokio::select! {
            _ = sigterm_stream.recv() => {
                info!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            source = shutdown_rx.recv() => {
                info!(
                    "received internal shutdown, performing graceful shutdown; source={:?}",
                    source,
                );
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            else => {
                // All other arms are closed, nothing left to do but return
                trace!("returning from graceful shutdown with all select arms closed");
            }
        };
    });

    Ok(graceful_shutdown_rx)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

fn faktory_job_wrapper(
    ctx_builder: Arc<DalContextBuilder>,
    job: Box<dyn JobConsumer + Send + Sync>,
    runtime: Arc<Runtime>,
) -> Result<(), io::Error> {
    info!("Execute {}: {job:?}", job.type_name());

    fn panic_wrapper(
        label: String,
        func: impl FnOnce() -> Result<(), TaskError>,
    ) -> Result<(), TaskError> {
        match std::panic::catch_unwind(AssertUnwindSafe(func)) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(any) => {
                // Note: Technically panicks can be of any form, but most should be &str or String
                match any.downcast::<String>() {
                    Ok(msg) => Err(TaskError::Panic(label, *msg)),
                    Err(any) => match any.downcast::<&str>() {
                        Ok(msg) => Err(TaskError::Panic(label, msg.to_string())),
                        Err(any) => {
                            let id = any.type_id();
                            error!("{label}: Panic message downcast failed of {id:?}",);
                            Err(TaskError::UnknownPanic(label, id))
                        }
                    },
                }
            }
        }
    }

    let outer_runtime = runtime.clone();
    let outer_ctx_builder = ctx_builder.clone();
    let access_builder = job.access_builder();
    let visibility = job.visibility();
    let job_type_name = job.type_name();
    let outer_type_name = job.type_name();

    let result = panic_wrapper(format!("Job execution: {}", job_type_name), move || {
        outer_runtime
            .block_on(async { job.run_job(outer_ctx_builder).await })
            .map_err(|err| TaskError::ExecutionFailed(outer_type_name, err))
    });

    if let Err(err) = result {
        error!("Job execution failed: {err}");

        let err_message = err.to_string();
        panic_wrapper(
            format!("Job failure reporting: {}", job_type_name),
            move || {
                runtime.block_on(async {
                    let mut txns = ctx_builder.transactions_starter().await?;
                    let txns = txns.start().await?;
                    let ctx = ctx_builder.build(access_builder.build(visibility), &txns);

                    JobFailure::new(&ctx, job_type_name.clone(), err_message).await?;

                    txns.commit().await?;
                    Ok(())
                })
            },
        )
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        return Err(io::Error::new(io::ErrorKind::Interrupted, err));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum TaskError {
    #[error("execution failed for {0}: {1}")]
    ExecutionFailed(String, JobConsumerError),
    #[error("{0} panicked: {1}")]
    Panic(String, String),
    #[error("{0} panicked with an unknown payload of {1:?}")]
    UnknownPanic(String, TypeId),
    #[error("failure reporting error: {0}")]
    FailureReporting(#[from] JobFailureError),
    #[error("pg error: {0}")]
    Pg(#[from] PgPoolError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}
