use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::server::config::JwtSecretKey;
use axum::routing::IntoMakeService;
use axum::Router;
use dal::{
    jwt_key::{install_new_jwt_key, jwt_key_exists},
    migrate, migrate_builtin_schemas, ResourceScheduler,
};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use si_data::{NatsClient, NatsConfig, NatsError, PgError, PgPool, PgPoolConfig, PgPoolError};
use telemetry::{prelude::*, TelemetryClient};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
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
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
}

pub type Result<T> = std::result::Result<T, ServerError>;

pub struct Server<I, S> {
    config: Config,
    inner: axum::Server<I, IntoMakeService<Router>>,
    socket: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server<(), ()> {
    pub fn http(
        config: Config,
        telemetry: telemetry::Client,
        pg_pool: PgPool,
        nats: NatsClient,
        veritech: veritech::Client,
        jwt_secret_key: JwtSecretKey,
    ) -> Result<Server<AddrIncoming, SocketAddr>> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx) =
                    build_service(telemetry, pg_pool, nats, veritech, jwt_secret_key)?;

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

    pub async fn uds(
        config: Config,
        telemetry: telemetry::Client,
        pg_pool: PgPool,
        nats: NatsClient,
        veritech: veritech::Client,
        jwt_secret_key: JwtSecretKey,
    ) -> Result<Server<UdsIncomingStream, PathBuf>> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx) =
                    build_service(telemetry, pg_pool, nats, veritech, jwt_secret_key)?;

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

    #[instrument(skip_all)]
    pub async fn generate_jwt_secret_key(path: impl AsRef<Path>) -> Result<JwtSecretKey> {
        JwtSecretKey::create(path).await.map_err(Into::into)
    }

    #[instrument(skip_all)]
    pub async fn load_jwt_secret_key(path: impl AsRef<Path>) -> Result<JwtSecretKey> {
        JwtSecretKey::load(path).await.map_err(Into::into)
    }

    #[instrument(skip_all)]
    pub async fn migrate_database(
        pg: &PgPool,
        nats: &NatsClient,
        jwt_secret_key: &JwtSecretKey,
        veritech: veritech::Client,
    ) -> Result<()> {
        migrate(pg).await?;
        migrate_builtin_schemas(pg, nats, veritech).await?;

        let mut conn = pg.get().await?;
        let txn = conn.transaction().await?;
        if !jwt_key_exists(&txn).await? {
            debug!("no jwt key found, generating new keypair");
            install_new_jwt_key(&txn, jwt_secret_key).await?;
        }
        txn.commit().await?;

        Ok(())
    }

    /// Start the basic resource sync scheduler
    pub async fn start_resource_sync_scheduler(
        pg: PgPool,
        nats: NatsClient,
        veritech: veritech::Client,
    ) {
        tokio::spawn(async move {
            let scheduler = ResourceScheduler::new(pg.clone(), nats.clone(), veritech.clone());
            scheduler.start().await;
        });
    }

    #[instrument(skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await.map_err(Into::into);
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        pool
    }

    #[instrument(skip_all)]
    pub async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await.map_err(Into::into);
        debug!("successfully connected nats client");
        client
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

pub fn build_service(
    telemetry: impl TelemetryClient,
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech::Client,
    jwt_secret_key: JwtSecretKey,
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
        veritech,
        jwt_secret_key,
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
