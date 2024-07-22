use axum::routing::IntoMakeService;
use axum::Router;
use dal::jwt_key::JwtConfig;
use dal::workspace_snapshot::migrator::{SnapshotGraphMigrator, SnapshotGraphMigratorError};
use dal::ServicesContext;
use dal::{BuiltinsError, JwtPublicSigningKey, TransactionsError, WorkspaceError};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use nats_multiplexer::Multiplexer;
use nats_multiplexer_client::MultiplexerClient;
use si_crypto::{
    SymmetricCryptoError, SymmetricCryptoService, SymmetricCryptoServiceConfig,
    VeritechCryptoConfig, VeritechEncryptionKey, VeritechEncryptionKeyError, VeritechKeyPairError,
};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolConfig, PgPoolError};
use si_pkg::SiPkgError;
use si_posthog::{PosthogClient, PosthogConfig};
use std::sync::Arc;
use std::{io, net::SocketAddr, path::Path, path::PathBuf};
use telemetry::prelude::*;
use telemetry_http::{HttpMakeSpan, HttpOnResponse};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal,
    sync::{broadcast, mpsc, oneshot},
    task::JoinError,
};
use tower_http::trace::TraceLayer;
use veritech_client::Client as VeritechClient;

use super::state::AppState;
use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};
use crate::server::config::VeritechKeyPair;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("intrinsics installation error: {0}")]
    Builtins(#[from] BuiltinsError),
    #[error(transparent)]
    DalInitialization(#[from] dal::InitializationError),
    #[error("error when loading veritech encryption key: {0}")]
    EncryptionKey(#[from] VeritechEncryptionKeyError),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("error initializing the server")]
    Init,
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error("jwt secret key error")]
    JwtSecretKey(#[from] dal::jwt_key::JwtKeyError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error(transparent)]
    Model(#[from] dal::ModelError),
    #[error("Module index: {0}")]
    ModuleIndex(#[from] module_index_client::ModuleIndexClientError),
    #[error("Module index url not set")]
    ModuleIndexNotSet,
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error("failed to install package")]
    PkgInstall,
    #[error(transparent)]
    Posthog(#[from] si_posthog::PosthogError),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error("snapshot migrator error: {0}")]
    SnapshotGraphMigrator(#[from] SnapshotGraphMigratorError),
    #[error(transparent)]
    SymmetricCryptoService(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Uds(#[from] UdsIncomingStreamError),
    #[error("Unable to parse URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("veritech public key already set")]
    VeritechPublicKeyAlreadySet,
    #[error("veritech public key error: {0}")]
    VeritechPublicKeyErr(#[from] VeritechKeyPairError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
}

impl From<PgPoolError> for ServerError {
    fn from(value: PgPoolError) -> Self {
        Self::PgPool(Box::new(value))
    }
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
        services_context: ServicesContext,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        ws_multiplexer: Multiplexer,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer: Multiplexer,
        crdt_multiplexer_client: MultiplexerClient,
    ) -> Result<(Server<AddrIncoming, SocketAddr>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    posthog_client,
                    ws_multiplexer_client,
                    crdt_multiplexer_client,
                )?;

                tokio::spawn(ws_multiplexer.run(shutdown_broadcast_rx.resubscribe()));
                tokio::spawn(crdt_multiplexer.run(shutdown_broadcast_rx.resubscribe()));

                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                let inner = axum::Server::bind(socket_addr).serve(service.into_make_service());
                let socket = inner.local_addr();

                Ok((
                    Server {
                        config,
                        inner,
                        socket,
                        shutdown_rx,
                    },
                    shutdown_broadcast_rx,
                ))
            }
            wrong @ IncomingStream::UnixDomainSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn uds(
        config: Config,
        services_context: ServicesContext,
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
        ws_multiplexer: Multiplexer,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer: Multiplexer,
        crdt_multiplexer_client: MultiplexerClient,
    ) -> Result<(Server<UdsIncomingStream, PathBuf>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    posthog_client,
                    ws_multiplexer_client,
                    crdt_multiplexer_client,
                )?;

                tokio::spawn(ws_multiplexer.run(shutdown_broadcast_rx.resubscribe()));
                tokio::spawn(crdt_multiplexer.run(shutdown_broadcast_rx.resubscribe()));

                info!("binding to Unix domain socket; path={}", path.display());
                let inner = axum::Server::builder(UdsIncomingStream::create(path).await?)
                    .serve(service.into_make_service());
                let socket = path.clone();

                Ok((
                    Server {
                        config,
                        inner,
                        socket,
                        shutdown_rx,
                    },
                    shutdown_broadcast_rx,
                ))
            }
            wrong @ IncomingStream::HTTPSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    pub fn init() -> Result<()> {
        Ok(dal::init()?)
    }

    pub async fn start_posthog(config: &PosthogConfig) -> Result<PosthogClient> {
        let (posthog_client, posthog_sender) = si_posthog::from_config(config)?;

        drop(tokio::spawn(posthog_sender.run()));

        Ok(posthog_client)
    }

    #[instrument(name = "sdf.init.generate_veritech_key_pair", level = "info", skip_all)]
    pub async fn generate_veritech_key_pair(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<()> {
        VeritechKeyPair::create_and_write_files(secret_key_path, public_key_path)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "sdf.init.generate_symmetric_key", level = "info", skip_all)]
    pub async fn generate_symmetric_key(symmetric_key_path: impl AsRef<Path>) -> Result<()> {
        SymmetricCryptoService::generate_key()
            .save(symmetric_key_path.as_ref())
            .await
            .map_err(Into::into)
    }

    #[instrument(
        name = "sdf.init.load_jwt_public_signing_key",
        level = "info",
        skip_all
    )]
    pub async fn load_jwt_public_signing_key(config: JwtConfig) -> Result<JwtPublicSigningKey> {
        Ok(JwtPublicSigningKey::from_config(config).await?)
    }

    #[instrument(
        name = "sdf.init.decode_jwt_public_signing_key",
        level = "info",
        skip_all
    )]
    pub async fn decode_jwt_public_signing_key(key_string: String) -> Result<JwtPublicSigningKey> {
        Ok(JwtPublicSigningKey::decode(key_string).await?)
    }

    #[instrument(name = "sdf.init.load_encryption_key", level = "info", skip_all)]
    pub async fn load_encryption_key(
        crypto_config: VeritechCryptoConfig,
    ) -> Result<Arc<VeritechEncryptionKey>> {
        Ok(Arc::new(
            VeritechEncryptionKey::from_config(crypto_config).await?,
        ))
    }

    pub async fn migrate_snapshots(services_context: &ServicesContext) -> Result<()> {
        let dal_context = services_context.clone().into_builder(true);
        let ctx = dal_context.build_default().await?;

        let mut migrator = SnapshotGraphMigrator::new();
        migrator.migrate_all(&ctx).await?;
        ctx.commit_no_rebase().await?;

        Ok(())
    }

    #[instrument(name = "sdf.init.migrate_database", level = "info", skip_all)]
    pub async fn migrate_database(services_context: &ServicesContext) -> Result<()> {
        services_context.layer_db().pg_migrate().await?;
        dal::migrate_all_with_progress(services_context).await?;

        Self::migrate_snapshots(services_context).await?;

        Ok(())
    }

    #[instrument(name = "sdf.init.create_pg_pool", level = "info", skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "sdf.init.connect_to_nats", level = "info", skip_all)]
    pub async fn connect_to_nats(nats_config: &NatsConfig) -> Result<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    pub fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(
        name = "sdf.init.create_symmetric_crypto_service",
        level = "info",
        skip_all
    )]
    pub async fn create_symmetric_crypto_service(
        config: &SymmetricCryptoServiceConfig,
    ) -> Result<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
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

pub fn build_service_for_tests(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    ws_multiplexer_client: MultiplexerClient,
    crdt_multiplexer_client: MultiplexerClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        posthog_client,
        true,
        ws_multiplexer_client,
        crdt_multiplexer_client,
    )
}

pub fn build_service(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    ws_multiplexer_client: MultiplexerClient,
    crdt_multiplexer_client: MultiplexerClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        posthog_client,
        false,
        ws_multiplexer_client,
        crdt_multiplexer_client,
    )
}

fn build_service_inner(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    for_tests: bool,
    ws_multiplexer_client: MultiplexerClient,
    crdt_multiplexer_client: MultiplexerClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let state = AppState::new(
        services_context,
        jwt_public_signing_key,
        posthog_client,
        shutdown_broadcast_tx.clone(),
        shutdown_tx,
        for_tests,
        ws_multiplexer_client,
        crdt_multiplexer_client,
    );

    let path_filter = Box::new(|path: &str| match path {
        "/api/" => Some(Level::TRACE),
        _ => None,
    });

    let routes = routes(state).layer(
        TraceLayer::new_for_http()
            .make_span_with(
                HttpMakeSpan::builder()
                    .level(Level::INFO)
                    .path_filter(path_filter)
                    .build(),
            )
            .on_response(HttpOnResponse::new().level(Level::DEBUG)),
    );

    let graceful_shutdown_rx = prepare_graceful_shutdown(shutdown_rx, shutdown_broadcast_tx)?;

    Ok((routes, graceful_shutdown_rx, shutdown_broadcast_rx))
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_watcher =
        signal::unix::signal(signal::unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

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
            _ = signal::ctrl_c() => {
                info!("received SIGINT signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx, shutdown_broadcast_tx);
            }
            _ = sigterm_watcher.recv() => {
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

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}
