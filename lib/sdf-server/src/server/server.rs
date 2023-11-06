use std::time::Duration;
use std::{io, net::SocketAddr, path::Path, path::PathBuf};

use axum::routing::IntoMakeService;
use axum::Router;
use hyper::server::{accept::Accept, conn::AddrIncoming};
use thiserror::Error;
use tokio::time::Instant;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal,
    sync::{broadcast, mpsc, oneshot},
    task::{JoinError, JoinSet},
    time,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use ulid::Ulid;

use dal::pkg::{import_pkg_from_pkg, ImportOptions, PkgError};
use dal::tasks::{StatusReceiver, StatusReceiverError};
use dal::{
    builtins, BuiltinsError, DalContext, JwtPublicSigningKey, Tenancy, TransactionsError,
    Workspace, WorkspaceError,
};
use dal::{tasks::ResourceScheduler, ServicesContext};
use module_index_client::types::BuiltinsDetailsResponse;
use module_index_client::{IndexClient, ModuleDetailsResponse};
use si_crypto::{
    CycloneKeyPairError, SymmetricCryptoError, SymmetricCryptoService, SymmetricCryptoServiceConfig,
};
use si_data_nats::{NatsClient, NatsConfig, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolConfig, PgPoolError};
use si_pkg::{SiPkg, SiPkgError};
use si_posthog::{PosthogClient, PosthogConfig};
use si_std::SensitiveString;
use telemetry::prelude::*;
use veritech_client::{Client as VeritechClient, CycloneEncryptionKey, CycloneEncryptionKeyError};

use crate::server::config::CycloneKeyPair;

use super::state::AppState;
use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("intrinsics installation error")]
    Builtins(#[from] BuiltinsError),
    #[error("cyclone public key already set")]
    CyclonePublicKeyAlreadySet,
    #[error("cyclone public key error: {0}")]
    CyclonePublicKeyErr(#[from] CycloneKeyPairError),
    #[error(transparent)]
    DalInitialization(#[from] dal::InitializationError),
    #[error("error when loading cyclone encryption key: {0}")]
    EncryptionKey(#[from] CycloneEncryptionKeyError),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("error initializing the server")]
    Init,
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error("jwt secret key error")]
    JwtSecretKey(#[from] dal::jwt_key::JwtKeyError),
    #[error(transparent)]
    Model(#[from] dal::ModelError),
    #[error("Module index: {0}")]
    ModuleIndex(#[from] module_index_client::IndexClientError),
    #[error("Module index url not set")]
    ModuleIndexNotSet,
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error("failed to install package")]
    PkgInstall,
    #[error(transparent)]
    Posthog(#[from] si_posthog::PosthogError),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    StatusReceiver(#[from] StatusReceiverError),
    #[error(transparent)]
    SymmetricCryptoService(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Uds(#[from] UdsIncomingStreamError),
    #[error("Unable to parse URL: {0}")]
    Url(#[from] url::ParseError),
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
    ) -> Result<(Server<AddrIncoming, SocketAddr>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    config.signup_secret().clone(),
                    posthog_client,
                )?;

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
    ) -> Result<(Server<UdsIncomingStream, PathBuf>, broadcast::Receiver<()>)> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
                    services_context,
                    jwt_public_signing_key,
                    config.signup_secret().clone(),
                    posthog_client,
                )?;

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

    #[instrument(name = "sdf.init.generate_cyclone_key_pair", skip_all)]
    pub async fn generate_cyclone_key_pair(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<()> {
        CycloneKeyPair::create_and_write_files(secret_key_path, public_key_path)
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "sdf.init.load_jwt_public_signing_key", skip_all)]
    pub async fn load_jwt_public_signing_key(
        path: impl AsRef<Path>,
    ) -> Result<JwtPublicSigningKey> {
        Ok(JwtPublicSigningKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.load_encryption_key", skip_all)]
    pub async fn load_encryption_key(path: impl AsRef<Path>) -> Result<CycloneEncryptionKey> {
        Ok(CycloneEncryptionKey::load(path).await?)
    }

    #[instrument(name = "sdf.init.migrate_database", skip_all)]
    pub async fn migrate_database(services_context: &ServicesContext) -> Result<()> {
        dal::migrate_all_with_progress(services_context).await?;
        migrate_builtins_from_module_index(services_context).await?;
        Ok(())
    }

    /// Start the basic resource refresh scheduler
    pub async fn start_resource_refresh_scheduler(
        services_context: ServicesContext,
        shutdown_broadcast_rx: broadcast::Receiver<()>,
    ) {
        ResourceScheduler::new(services_context).start(shutdown_broadcast_rx);
    }

    pub async fn start_status_updater(
        services_context: ServicesContext,
        shutdown_broadcast_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        StatusReceiver::new(services_context)
            .await?
            .start(shutdown_broadcast_rx);
        Ok(())
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

    pub fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "sdf.init.create_symmetric_crypto_service", skip_all)]
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

pub async fn migrate_builtins_from_module_index(services_context: &ServicesContext) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(5));
    let instant = Instant::now();

    let mut dal_context = services_context.clone().into_builder(true);
    dal_context.set_no_dependent_values();
    let mut ctx = dal_context.build_default().await?;

    let workspace = Workspace::builtin(&ctx).await?;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));
    ctx.blocking_commit().await?;

    info!("migrating intrinsic functions");
    builtins::func::migrate_intrinsics(&ctx).await?;
    info!("migrating builtin functions");
    builtins::func::migrate(&ctx).await?;

    let module_index_url = services_context
        .module_index_url()
        .as_ref()
        .ok_or(ServerError::ModuleIndexNotSet)?;

    let module_index_client =
        IndexClient::unauthenticated_client(module_index_url.clone().as_str().try_into()?);
    let module_list = module_index_client.list_builtins().await?;
    let install_builtins = install_builtins(ctx, module_list, module_index_client);
    tokio::pin!(install_builtins);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!(elapsed = instant.elapsed().as_secs_f32(), "migrating");
            }
            result = &mut install_builtins  => match result {
                Ok(_) => {
                    info!(elapsed = instant.elapsed().as_secs_f32(), "migrating completed");
                    break;
                }
                Err(err) => return Err(err),
            }
        }
    }

    Ok(())
}

async fn install_builtins(
    ctx: DalContext,
    module_list: BuiltinsDetailsResponse,
    module_index_client: IndexClient,
) -> Result<()> {
    let dal = &ctx;
    let client = &module_index_client.clone();
    let modules = module_list.modules;
    let total = modules.len();

    let mut join_set = JoinSet::new();
    for module in modules {
        let module = module.clone();
        let client = client.clone();
        join_set.spawn(async move {
            (
                module.name.to_owned(),
                fetch_builtin(&module, &client).await,
            )
        });
    }

    let mut count: usize = 0;
    while let Some(res) = join_set.join_next().await {
        let (pkg_name, res) = res?;
        match res {
            Ok(pkg) => {
                if let Err(err) = import_pkg_from_pkg(
                    &ctx,
                    &pkg,
                    Some(ImportOptions {
                        schemas: None,
                        skip_import_funcs: None,
                        no_record: false,
                        is_builtin: true,
                    }),
                )
                .await
                {
                    println!("Pkg {pkg_name} Install failed, {err}");
                } else {
                    ctx.commit().await?;

                    count += 1;
                    println!(
                        "Pkg {pkg_name} Install finished successfully. {count} of {total} installed.",
                    );
                }
            }
            Err(err) => {
                println!("Pkg {pkg_name} Install failed, {err}");
            }
        }
    }

    dal.commit().await?;

    Ok(())
}

async fn fetch_builtin(
    module: &ModuleDetailsResponse,
    module_index_client: &IndexClient,
) -> Result<SiPkg> {
    let module = module_index_client
        .get_builtin(Ulid::from_string(module.id.as_str()).unwrap_or(Ulid::new()))
        .await?;

    Ok(SiPkg::load_from_bytes(module)?)
}

pub fn build_service_for_tests(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        signup_secret,
        posthog_client,
        true,
    )
}

pub fn build_service(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    build_service_inner(
        services_context,
        jwt_public_signing_key,
        signup_secret,
        posthog_client,
        false,
    )
}

fn build_service_inner(
    services_context: ServicesContext,
    jwt_public_signing_key: JwtPublicSigningKey,
    signup_secret: SensitiveString,
    posthog_client: PosthogClient,
    for_tests: bool,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let state = AppState::new(
        services_context,
        signup_secret,
        jwt_public_signing_key,
        posthog_client,
        shutdown_broadcast_tx.clone(),
        shutdown_tx,
        for_tests,
    );

    let routes = routes(state)
        // TODO(fnichol): customize http tracing further, using:
        // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
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
