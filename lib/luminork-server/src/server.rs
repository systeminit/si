use std::{
    fmt,
    future::IntoFuture as _,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

use audit_database::AuditDatabaseContext;
use axum::{
    Router,
    async_trait,
    routing::IntoMakeService,
};
use dal::ServicesContext;
use edda_client::EddaClient;
use frigg::{
    FriggStore,
    frigg_kv,
};
use hyper::server::accept::Accept;
use nats_multiplexer::Multiplexer;
use nats_multiplexer_client::MultiplexerClient;
use sdf_core::nats_multiplexer::EddaUpdatesMultiplexerClient;
use si_data_nats::jetstream;
use si_data_spicedb::SpiceDbClient;
use si_jwt_public_key::JwtPublicSigningKeyChain;
use si_posthog::PosthogClient;
use telemetry::prelude::*;
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
    },
    signal,
    sync::RwLock,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    ApplicationRuntimeMode,
    AxumApp,
    Config,
    IncomingStream,
    ServerError,
    ServerResult,
    WorkspacePermissions,
    WorkspacePermissionsMode,
    init,
    nats_multiplexer::{
        CRDT_MULTIPLEXER_SUBJECT,
        WS_MULTIPLEXER_SUBJECT,
    },
    runnable::Runnable,
    uds::UdsIncomingStream,
};

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    instance_id: String,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

pub struct Server {
    metadata: Arc<ServerMetadata>,
    inner: Box<dyn Runnable + Send>,
    socket: ServerSocket,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("metadata", &self.metadata)
            .field("socket", &self.socket)
            .finish()
    }
}

#[async_trait]
impl Runnable for Server {
    async fn try_run(self) -> ServerResult<()> {
        self.inner.try_run().await?;
        info!("luminork main loop shutdown complete");
        Ok(())
    }
}

impl Server {
    #[instrument(name = "luminork.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        token: CancellationToken,
        helping_tasks_tracker: &TaskTracker,
        helping_tasks_token: CancellationToken,
    ) -> ServerResult<Self> {
        let (services_context, layer_db_graceful_shutdown) =
            init::services_context_from_config(&config, helping_tasks_token.clone()).await?;

        let jwt_public_signing_key = init::load_jwt_public_signing_key(
            config.jwt_signing_public_key().clone(),
            config.jwt_secondary_signing_public_key().cloned(),
        )
        .await?;
        let (posthog_sender, posthog_client) =
            init::initialize_posthog(config.posthog(), helping_tasks_token.clone())?;

        let (ws_multiplexer, ws_multiplexer_client) = Multiplexer::new(
            services_context.nats_conn(),
            WS_MULTIPLEXER_SUBJECT,
            helping_tasks_token.clone(),
        )
        .await?;
        let (crdt_multiplexer, crdt_multiplexer_client) = Multiplexer::new(
            services_context.nats_conn(),
            CRDT_MULTIPLEXER_SUBJECT,
            helping_tasks_token.clone(),
        )
        .await?;
        let (edda_updates_multiplexer, edda_updates_multiplexer_client) = Multiplexer::new(
            services_context.nats_conn(),
            edda_core::nats::subject::all_workspace_updates_for_all_workspaces(
                services_context.nats_conn().metadata().subject_prefix(),
            ),
            helping_tasks_token.clone(),
        )
        .await?;
        let edda_updates_multiplexer_client =
            EddaUpdatesMultiplexerClient::new(edda_updates_multiplexer_client);

        let application_runtime_mode = Arc::new(RwLock::new(ApplicationRuntimeMode::Running));

        let mut spicedb_client = None;
        if config.spicedb().enabled {
            spicedb_client = Some(SpiceDbClient::new(config.spicedb()).await?);
        }

        let frigg = {
            let nats = services_context.nats_conn().clone();
            let context = jetstream::new(nats.clone());

            FriggStore::new(
                nats,
                frigg_kv(&context, context.metadata().subject_prefix()).await?,
            )
        };

        prepare_maintenance_mode_watcher(application_runtime_mode.clone(), token.clone())?;

        // Spawn helping tasks and track them for graceful shutdown
        helping_tasks_tracker.spawn(layer_db_graceful_shutdown.into_future());
        helping_tasks_tracker.spawn(posthog_sender.run());
        helping_tasks_tracker.spawn(ws_multiplexer.run());
        helping_tasks_tracker.spawn(crdt_multiplexer.run());
        helping_tasks_tracker.spawn(edda_updates_multiplexer.run());

        let audit_database_context = AuditDatabaseContext::from_config(config.audit()).await?;

        let edda_client =
            edda_client::EddaClient::new(services_context.nats_conn().clone()).await?;

        Self::from_services(
            config.instance_id().to_string(),
            config.incoming_stream().clone(),
            services_context,
            jwt_public_signing_key,
            posthog_client,
            config.auth_api_url(),
            ws_multiplexer_client,
            crdt_multiplexer_client,
            edda_updates_multiplexer_client,
            *config.create_workspace_permissions(),
            config.create_workspace_allowlist().clone(),
            application_runtime_mode,
            token,
            spicedb_client,
            frigg,
            audit_database_context,
            edda_client,
        )
        .await
    }

    #[instrument(name = "luminork.init.from_services", level = "info", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn from_services(
        instance_id: impl Into<String>,
        incoming_stream: IncomingStream,
        services_context: ServicesContext,
        jwt_public_signing_key_chain: JwtPublicSigningKeyChain,
        posthog_client: PosthogClient,
        auth_api_url: impl AsRef<str>,
        ws_multiplexer_client: MultiplexerClient,
        crdt_multiplexer_client: MultiplexerClient,
        edda_updates_multiplexer_client: EddaUpdatesMultiplexerClient,
        create_workspace_permissions: WorkspacePermissionsMode,
        create_workspace_allowlist: Vec<WorkspacePermissions>,
        application_runtime_mode: Arc<RwLock<ApplicationRuntimeMode>>,
        token: CancellationToken,
        spicedb_client: Option<SpiceDbClient>,
        frigg: FriggStore,
        audit_database_context: AuditDatabaseContext,
        edda_client: EddaClient,
    ) -> ServerResult<Self> {
        let app = AxumApp::from_services(
            services_context.clone(),
            jwt_public_signing_key_chain,
            posthog_client,
            auth_api_url,
            ws_multiplexer_client,
            crdt_multiplexer_client,
            edda_updates_multiplexer_client,
            create_workspace_permissions,
            create_workspace_allowlist,
            application_runtime_mode,
            token.clone(),
            spicedb_client,
            frigg,
            audit_database_context.clone(),
            edda_client,
        )
        .into_inner();

        let (inner, socket): (Box<dyn Runnable + Send>, _) = match incoming_stream {
            IncomingStream::TcpSocket(socket_addr) => {
                debug!(%socket_addr, "binding to tcp socket");
                let inner = axum::Server::bind(&socket_addr).serve(app.into_make_service());
                let socket = inner.local_addr();
                info!(%socket, "http service bound to tcp socket");

                (
                    Box::new(InnerServer { inner, token }),
                    ServerSocket::SocketAddr(socket),
                )
            }
            IncomingStream::UnixDomainSocket(path) => {
                debug!(path = %path.display(), "binding to unix domain socket");
                let inner = axum::Server::builder(UdsIncomingStream::create(&path).await?)
                    .serve(app.into_make_service());
                let socket = path;
                info!(socket = %socket.display(), "http service bound to unix domain socket");

                (
                    Box::new(InnerServer { inner, token }),
                    ServerSocket::DomainSocket(socket),
                )
            }
        };

        let metadata = Arc::new(ServerMetadata {
            instance_id: instance_id.into(),
        });

        Ok(Self {
            metadata,
            inner,
            socket,
        })
    }

    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running luminork main loop");
        }
    }
}

#[derive(Debug)]
#[remain::sorted]
pub enum ServerSocket {
    DomainSocket(PathBuf),
    SocketAddr(SocketAddr),
}

struct InnerServer<I> {
    inner: axum::Server<I, IntoMakeService<Router>>,
    token: CancellationToken,
}

#[async_trait]
impl<I, IO, IE> Runnable for InnerServer<I>
where
    I: Accept<Conn = IO, Error = IE> + Send + Sync,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    async fn try_run(self) -> ServerResult<()> {
        let token = self.token;

        self.inner
            .with_graceful_shutdown(async {
                token.cancelled().await;
            })
            .await
            .map_err(ServerError::Axum)
    }
}

fn prepare_maintenance_mode_watcher(
    mode: Arc<RwLock<ApplicationRuntimeMode>>,
    cancellation_token: CancellationToken,
) -> ServerResult<()> {
    let mut sigusr2_watcher = signal::unix::signal(signal::unix::SignalKind::user_defined2())
        .map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = sigusr2_watcher.recv() => {
                    info!("received SIGUSR2 signal, changing application runtime mode");
                    let mut mode = mode.write().await;
                    info!(?mode, "current application runtime mode (changing it...)");
                    *mode = match *mode {
                        ApplicationRuntimeMode::Maintenance => ApplicationRuntimeMode::Running,
                        ApplicationRuntimeMode::Running => ApplicationRuntimeMode::Maintenance,
                    };
                    info!(?mode, "new application runtime mode (changed!)");
                }
                _ = cancellation_token.cancelled() => {
                    break
                }
                else => {
                    // All other arms are closed, nothing left to do but return
                    trace!("returning from graceful shutdown with all select arms closed");
                    break
                }
            }
        }
    });

    Ok(())
}
