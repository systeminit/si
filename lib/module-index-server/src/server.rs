use std::{
    io,
    net::SocketAddr,
    time::Duration,
};

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    response::{
        IntoResponse,
        Response,
    },
    routing::IntoMakeService,
};
use hyper::{
    StatusCode,
    server::{
        accept::Accept,
        conn::AddrIncoming,
    },
};
use s3::creds::{
    Credentials as AwsCredentials,
    error::CredentialsError,
};
use sea_orm::{
    ConnectOptions,
    Database,
    DatabaseConnection,
    DbErr,
};
use si_data_pg::{
    PgPool,
    PgPoolConfig,
    PgPoolError,
};
use si_jwt_public_key::{
    JwtConfig,
    JwtPublicSigningKeyChain,
    JwtPublicSigningKeyError,
};
use si_posthog::{
    PosthogClient,
    PosthogConfig,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
    },
    signal,
    sync::{
        broadcast,
        mpsc,
        oneshot,
    },
};
use tokio_util::sync::CancellationToken;
use tower::{
    BoxError,
    ServiceBuilder,
    buffer::BufferLayer,
    limit::RateLimitLayer,
};
use tower_http::trace::{
    DefaultMakeSpan,
    TraceLayer,
};

use super::routes;
use crate::{
    Config,
    app_state::{
        AppState,
        ShutdownSource,
    },
    config::RateLimitConfig,
    s3::S3Config,
};

mod embedded_migrations {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("bad aws config")]
    AwsConfigError,
    #[error("aws creds error: {0}")]
    CredentialsError(#[from] CredentialsError),
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("jwt public key error: {0}")]
    JwtPublicKey(#[from] JwtPublicSigningKeyError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] Box<PgPoolError>),
    #[error("posthog error: {0}")]
    Posthog(#[from] si_posthog::PosthogError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
}

impl From<PgPoolError> for ServerError {
    fn from(e: PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}

type ServerResult<T> = std::result::Result<T, ServerError>;

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
        pg_pool: DatabaseConnection,
        jwt_public_signing_key: JwtPublicSigningKeyChain,
        posthog_client: PosthogClient,
    ) -> ServerResult<(Server<AddrIncoming, SocketAddr>, broadcast::Receiver<()>)> {
        // socket_addr

        // try to load aws creds from a few different places
        let aws_creds = match (&config.s3().access_key_id, &config.s3().secret_access_key) {
            (Some(aws_key), Some(aws_secret)) => {
                AwsCredentials::new(Some(aws_key), Some(aws_secret), None, None, None)?
            }
            (None, None) => match AwsCredentials::from_env() {
                Ok(creds) => creds,
                Err(CredentialsError::MissingEnvVar(_, _)) => {
                    // Attempt to load from local AWS Profile
                    info!("could not load credentials from environment; falling back to profile");
                    match AwsCredentials::from_profile(None) {
                        Ok(creds) => creds,
                        Err(err) => {
                            info!(
                                ?err,
                                "could not load credentials from profile; falling back to instance metadata"
                            );

                            // Attempt to load from instance metadata
                            match AwsCredentials::from_instance_metadata() {
                                Ok(creds) => creds,
                                Err(err) => return Err(err.into()),
                            }
                        }
                    }
                }
                Err(err) => return Err(err.into()),
            },
            _ => {
                return Err(ServerError::AwsConfigError);
            }
        };

        let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
            pg_pool,
            config.auth_api_url().to_owned(),
            jwt_public_signing_key,
            posthog_client,
            aws_creds,
            config.rate_limit().clone(),
            config.s3().clone(),
        )?;

        info!(
            "binding to HTTP socket; socket_addr={}",
            config.socket_addr()
        );
        let inner = axum::Server::bind(config.socket_addr()).serve(service.into_make_service());
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

    // this creates our si_data_pg::PgPool, which wont work with SeaORM
    #[instrument(name = "module-index.init.create_pg_pool", level = "info", skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> ServerResult<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    // this creates the sea-orm managed db connection (also a pool)
    #[instrument(
        name = "module-index.init.create_db_connection",
        level = "info",
        skip_all
    )]
    pub async fn create_db_connection(
        pg_pool_config: &PgPoolConfig,
    ) -> ServerResult<DatabaseConnection> {
        let mut opt = ConnectOptions::new(format!(
            "{protocol}://{username}:{password}@{host}:{port}/{database}",
            protocol = "postgres",
            username = pg_pool_config.user,
            password = Into::<String>::into(pg_pool_config.password.clone()),
            host = pg_pool_config.hostname,
            port = pg_pool_config.port,
            database = pg_pool_config.dbname
        ));

        opt.max_connections(pg_pool_config.pool_max_size.try_into().unwrap())
            .min_connections(5);
        if let Some(timeout) = pg_pool_config.pool_timeout_create_secs {
            opt.connect_timeout(Duration::from_secs(timeout));
        }
        if let Some(timeout) = pg_pool_config.pool_timeout_wait_secs {
            opt.acquire_timeout(Duration::from_secs(timeout));
        }
        if let Some(timeout) = pg_pool_config.pool_timeout_recycle_secs {
            opt.idle_timeout(Duration::from_secs(timeout));
        }

        let db = Database::connect(opt).await?;
        debug!("successfully created db connection pool");
        Ok(db)
    }

    pub async fn run_migrations(pg_pool: &PgPool) -> ServerResult<()> {
        Ok(pg_pool
            .migrate(embedded_migrations::migrations::runner())
            .await?)
    }

    #[instrument(
        name = "sdf.init.load_jwt_public_signing_key",
        level = "info",
        skip_all
    )]
    pub async fn load_jwt_public_signing_key(
        config: &Config,
    ) -> ServerResult<JwtPublicSigningKeyChain> {
        let primary = JwtConfig {
            key_file: Some(config.jwt_signing_public_key_path().to_owned()),
            key_base64: None,
            algo: config.jwt_signing_public_key_algo(),
        };

        let secondary = config
            .jwt_secondary_signing_public_key_path()
            .zip(config.jwt_secondary_signing_public_key_algo())
            .map(|(path, algo)| JwtConfig {
                key_file: Some(path.to_owned()),
                key_base64: None,
                algo,
            });

        Ok(JwtPublicSigningKeyChain::from_config(primary, secondary).await?)
    }

    pub async fn start_posthog(config: &PosthogConfig) -> ServerResult<PosthogClient> {
        // TODO(fnichol): this should be threaded through
        let token = CancellationToken::new();

        let (posthog_sender, posthog_client) = si_posthog::from_config(config, token)?;

        drop(tokio::spawn(posthog_sender.run()));

        Ok(posthog_client)
    }
}

impl<I, IO, IE, S> Server<I, S>
where
    I: Accept<Conn = IO, Error = IE>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn run(self) -> ServerResult<()> {
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
    pg_pool: DatabaseConnection,
    auth_api_url: String,
    jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
    posthog_client: PosthogClient,
    aws_creds: AwsCredentials,
    rate_limit_config: RateLimitConfig,
    s3_config: S3Config,
) -> ServerResult<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let state = AppState::new(
        pg_pool,
        auth_api_url,
        jwt_public_signing_key_chain,
        posthog_client,
        aws_creds,
        s3_config,
        shutdown_tx,
    );

    let routes = routes::routes(state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    tracing::error!(error = %err, "Unexpected error in request processing");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Internal server error: {err}"))
                        .expect("Unable to build error response body")
                        .into_response()
                }))
                .layer(BufferLayer::new(128))
                .layer(Into::<RateLimitLayer>::into(rate_limit_config)),
        )
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
) -> ServerResult<oneshot::Receiver<()>> {
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
