use std::{io, net::SocketAddr, path::Path, time::Duration};

use super::routes;

use axum::routing::IntoMakeService;
use axum::Router;
use hyper::server::{accept::Accept, conn::AddrIncoming};
use s3::creds::{error::CredentialsError, Credentials as AwsCredentials};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use si_posthog::{PosthogClient, PosthogConfig};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal,
    sync::{broadcast, mpsc, oneshot},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{
    app_state::{AppState, ShutdownSource},
    jwt_key::{JwtKeyError, JwtPublicSigningKey},
    s3::S3Config,
    Config,
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
    #[error("jwt secret key error")]
    JwtSecretKey(#[from] JwtKeyError),
    #[error(transparent)]
    PgPool(#[from] Box<PgPoolError>),
    #[error(transparent)]
    Posthog(#[from] si_posthog::PosthogError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
}

impl From<PgPoolError> for ServerError {
    fn from(e: PgPoolError) -> Self {
        Self::PgPool(Box::new(e))
    }
}

type Result<T> = std::result::Result<T, ServerError>;

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
        jwt_public_signing_key: JwtPublicSigningKey,
        posthog_client: PosthogClient,
    ) -> Result<(Server<AddrIncoming, SocketAddr>, broadcast::Receiver<()>)> {
        // socket_addr

        // try to load aws creds from a few different places
        let aws_creds = match (&config.s3().access_key_id, &config.s3().secret_access_key) {
            (Some(aws_key), Some(aws_secret)) => {
                AwsCredentials::new(Some(aws_key), Some(aws_secret), None, None, None)?
            }
            (None, None) => match AwsCredentials::from_env() {
                Ok(creds) => creds,
                Err(CredentialsError::MissingEnvVar(_, _)) => AwsCredentials::from_profile(None)?,
                Err(err) => return Err(err.into()),
            },
            _ => {
                return Err(ServerError::AwsConfigError);
            }
        };

        let (service, shutdown_rx, shutdown_broadcast_rx) = build_service(
            pg_pool,
            jwt_public_signing_key,
            posthog_client,
            aws_creds,
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
    #[instrument(name = "module-index.init.create_pg_pool", skip_all)]
    pub async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> Result<PgPool> {
        let pool = PgPool::new(pg_pool_config).await?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    // this creates the sea-orm managed db connection (also a pool)
    #[instrument(name = "module-index.init.create_db_connection", skip_all)]
    pub async fn create_db_connection(pg_pool_config: &PgPoolConfig) -> Result<DatabaseConnection> {
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

    pub async fn run_migrations(pg_pool: &PgPool) -> Result<()> {
        Ok(pg_pool
            .migrate(embedded_migrations::migrations::runner())
            .await?)
    }

    #[instrument(name = "sdf.init.load_jwt_public_signing_key", skip_all)]
    pub async fn load_jwt_public_signing_key(
        path: impl AsRef<Path>,
    ) -> Result<JwtPublicSigningKey> {
        Ok(JwtPublicSigningKey::load(path).await?)
    }

    pub async fn start_posthog(config: &PosthogConfig) -> Result<PosthogClient> {
        let (posthog_client, posthog_sender) = si_posthog::from_config(config)?;

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
    pg_pool: DatabaseConnection,
    jwt_public_signing_key: JwtPublicSigningKey,
    posthog_client: PosthogClient,
    aws_creds: AwsCredentials,
    s3_config: S3Config,
) -> Result<(Router, oneshot::Receiver<()>, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let state = AppState::new(
        pg_pool,
        jwt_public_signing_key,
        posthog_client,
        aws_creds,
        s3_config,
        shutdown_broadcast_tx.clone(),
        shutdown_tx,
    );

    let routes = routes::routes(state)
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
