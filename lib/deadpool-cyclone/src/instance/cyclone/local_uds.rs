use std::{
    io,
    path::{Path, PathBuf},
    result,
    time::Duration,
};

use async_trait::async_trait;
use cyclone::{
    canonical_command::CanonicalCommand,
    client::{
        Connection, PingExecution, QualificationCheckExecution, ResolverFunctionExecution,
        ResourceSyncExecution, UnixStream, Watch, WatchError, WatchStarted,
    },
    process::{self, ShutdownError},
    Client, ClientError, CycloneClient, LivenessStatus, QualificationCheckRequest, ReadinessStatus,
    ResolverFunctionRequest, ResourceSyncRequest, UdsClient,
};
use derive_builder::Builder;
use futures::StreamExt;
use tempfile::{NamedTempFile, TempPath};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::{Child, Command},
    sync::oneshot,
    time,
};
use tracing::{debug, trace, warn};

use crate::instance::{Instance, Spec, SpecBuilder};

/// Error type for [`LocalUdsInstance`].
#[derive(Debug, Error)]
pub enum LocalUdsInstanceError {
    /// Spec builder error.
    #[error(transparent)]
    Builder(#[from] LocalUdsInstanceSpecBuilderError),
    /// Failed to spawn a child process.
    #[error("failed to spawn cyclone child process")]
    ChildSpawn(#[source] io::Error),
    /// Error when waiting for child process to shutdown.
    #[error(transparent)]
    ChildShutdown(#[from] ShutdownError),
    /// Cyclone client error.
    #[error(transparent)]
    Client(#[from] ClientError),
    /// Instance has exhausted its predefined request count.
    #[error("no remaining requests, cyclone server is considered unhealthy")]
    NoRemainingRequests,
    /// Failed to create socket from temporary file.
    #[error("failed to create temp socket")]
    TempSocket(#[source] io::Error),
    /// Cyclone client `watch` endpoint error.
    #[error(transparent)]
    Watch(#[from] WatchError),
    /// Cyclone client `watch` session ended earlier than expected.
    #[error("server closed watch session before expected")]
    WatchClosed,
    /// Cyclone client initial `watch` session connection with retries timed out.
    #[error("timeout while retrying to start a client watch session")]
    WatchInitTimeout,
    /// Cyclone client `watch` session shut down earlier than expected.
    #[error("watch session is shut down, cyclone server is considered unhealthy")]
    WatchShutDown,
}

type Result<T> = result::Result<T, LocalUdsInstanceError>;

/// A local Cyclone [`Instance`], managed as a spawned child process, communicating over a Unix
/// domain socket.
#[derive(Debug)]
pub struct LocalUdsInstance {
    // The `TempPath` type is kept around as an [RAII
    // guard](https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html), that is,
    // when `LocalUdsInstance` is dropped, the temp file is marked for deletion.
    _temp_path: Option<TempPath>,
    client: UdsClient,
    limit_requests: Option<u32>,
    child: Child,
    watch_shutdown_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Instance for LocalUdsInstance {
    type SpecBuilder = LocalUdsInstanceSpecBuilder;
    type Error = LocalUdsInstanceError;

    async fn terminate(mut self) -> result::Result<(), Self::Error> {
        if !self.watch_shutdown_tx.is_closed() && self.watch_shutdown_tx.send(()).is_err() {
            debug!("sent watch shutdown but receiver was already closed");
        }
        process::child_shutdown(&mut self.child, Some(process::Signal::SIGTERM), None).await?;

        Ok(())
    }

    async fn ensure_healthy(&mut self) -> result::Result<(), Self::Error> {
        self.ensure_healthy_client().await?;
        match self.client.readiness().await? {
            ReadinessStatus::Ready => {}
        }

        Ok(())
    }
}

#[async_trait]
impl CycloneClient<UnixStream> for LocalUdsInstance {
    async fn watch(&mut self) -> result::Result<Watch<UnixStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        self.client.watch().await
    }

    async fn liveness(&mut self) -> result::Result<LivenessStatus, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        self.client.liveness().await
    }

    async fn readiness(&mut self) -> result::Result<ReadinessStatus, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        self.client.readiness().await
    }

    async fn execute_ping(&mut self) -> result::Result<PingExecution<UnixStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_ping().await;
        self.count_request();

        result
    }

    async fn execute_qualification(
        &mut self,
        request: QualificationCheckRequest,
    ) -> result::Result<QualificationCheckExecution<UnixStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_qualification(request).await;
        self.count_request();

        result
    }

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> result::Result<ResolverFunctionExecution<UnixStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_resolver(request).await;
        self.count_request();

        result
    }

    async fn execute_sync(
        &mut self,
        request: ResourceSyncRequest,
    ) -> result::Result<ResourceSyncExecution<UnixStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_sync(request).await;
        self.count_request();

        result
    }
}

impl LocalUdsInstance {
    async fn ensure_healthy_client(&mut self) -> Result<()> {
        if !self.is_watch_shutdown_open() {
            return Err(LocalUdsInstanceError::WatchShutDown);
        }
        if !self.has_remaining_requests() {
            return Err(LocalUdsInstanceError::NoRemainingRequests);
        }

        Ok(())
    }

    fn has_remaining_requests(&self) -> bool {
        match self.limit_requests {
            Some(remaining) if remaining == 0 => false,
            Some(_) | None => true,
        }
    }

    fn is_watch_shutdown_open(&self) -> bool {
        !self.watch_shutdown_tx.is_closed()
    }

    fn count_request(&mut self) {
        if let Some(limit_requests) = self.limit_requests.as_mut() {
            *limit_requests = limit_requests.saturating_sub(1);
        }
    }
}

/// The [`Spec`] for [`LocalUdsInstance`]
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
pub struct LocalUdsInstanceSpec {
    /// Canonical path to the `cyclone` program.
    #[builder(try_setter, setter(into))]
    cyclone_cmd_path: CanonicalCommand,

    /// Canonical path to the language server program.
    #[builder(try_setter, setter(into))]
    lang_server_cmd_path: CanonicalCommand,

    /// Socket strategy for a spawned Cyclone server.
    #[builder(default)]
    socket_strategy: LocalUdsSocketStrategy,

    /// Sets the watch timeout value for a spawned Cyclone server.
    #[builder(setter(into, strip_option), default)]
    watch_timeout: Option<Duration>,

    /// Sets the limit requests strategy for a spawned Cyclone server.
    #[builder(setter(into), default = "Some(1)")]
    limit_requests: Option<u32>,

    /// Enables the `ping` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_ping"), default = "false")]
    ping: bool,

    /// Enables the `qualification` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_qualification"), default = "false")]
    qualification: bool,

    /// Enables the `resolver` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_resolver"), default = "false")]
    resolver: bool,

    /// Enables the `sync` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_sync"), default = "false")]
    sync: bool,
}

#[async_trait]
impl Spec for LocalUdsInstanceSpec {
    type Instance = LocalUdsInstance;
    type Error = LocalUdsInstanceError;

    async fn spawn(&self) -> result::Result<Self::Instance, Self::Error> {
        let (temp_path, socket) = temp_path_and_socket_from(&self.socket_strategy)?;
        let mut cmd = self.build_command(&socket);

        debug!("spawning child process; cmd={:?}", &cmd);
        let child = cmd.spawn().map_err(Self::Error::ChildSpawn)?;

        let mut client = Client::uds(socket)?;

        // Establish the client watch session. As the process may be booting, we will retry for a
        // period before giving up and assuming that the server instance has failed.
        let watch = {
            let mut retries = 30;
            loop {
                trace!("calling client.watch()");
                if let Ok(watch) = client.watch().await {
                    trace!("client watch session established");
                    break watch;
                }
                if retries < 1 {
                    return Err(Self::Error::WatchInitTimeout);
                }
                retries -= 1;
                time::sleep(Duration::from_millis(64)).await;
            }
        };

        let mut watch_progress = watch.start().await?;
        // Establish that we have received our first watch ping, which should happen immediately
        // after establishing a watch session
        watch_progress
            .next()
            .await
            .ok_or(Self::Error::WatchClosed)??;

        let (watch_shutdown_tx, watch_shutdown_rx) = oneshot::channel();
        // Spawn a task to keep the watch session open until we shut it down
        tokio::spawn(watch_task(watch_progress, watch_shutdown_rx));

        Ok(Self::Instance {
            _temp_path: temp_path,
            client,
            limit_requests: self.limit_requests,
            child,
            watch_shutdown_tx,
        })
    }
}

impl LocalUdsInstanceSpec {
    fn build_command(&self, socket: &Path) -> Command {
        let mut cmd = Command::new(&self.cyclone_cmd_path);
        cmd.arg("--bind-uds")
            .arg(&socket)
            .arg("--lang-server")
            .arg(&self.lang_server_cmd_path)
            .arg("--enable-watch");
        if let Some(limit_requests) = self.limit_requests {
            cmd.arg("--limit-requests").arg(limit_requests.to_string());
        }
        if let Some(timeout) = self.watch_timeout {
            cmd.arg("--watch-timeout")
                .arg(timeout.as_secs().to_string());
        }
        if self.ping {
            cmd.arg("--enable-ping");
        }
        if self.qualification {
            cmd.arg("--enable-qualification");
        }
        if self.resolver {
            cmd.arg("--enable-resolver");
        }
        if self.sync {
            cmd.arg("--enable-sync");
        }

        cmd
    }
}

impl SpecBuilder for LocalUdsInstanceSpecBuilder {
    type Spec = LocalUdsInstanceSpec;
    type Error = LocalUdsInstanceError;

    fn build(&self) -> result::Result<Self::Spec, Self::Error> {
        self.build().map_err(Into::into)
    }
}

impl LocalUdsInstanceSpecBuilder {
    /// Sets the limit requests strategy to `1` for a spawned Cyclone server.
    pub fn oneshot(&mut self) -> &mut Self {
        self.limit_requests(Some(1))
    }

    /// Enables the `ping` execution endpoint for a spawned Cyclone server.
    pub fn ping(&mut self) -> &mut Self {
        self._ping(true)
    }

    /// Enables the `qualification` execution endpoint for a spawned Cyclone server.
    pub fn qualification(&mut self) -> &mut Self {
        self._qualification(true)
    }

    /// Enables the `resolver` execution endpoint for a spawned Cyclone server.
    pub fn resolver(&mut self) -> &mut Self {
        self._resolver(true)
    }

    /// Enables the `sync` execution endpoint for a spawned Cyclone server.
    pub fn sync(&mut self) -> &mut Self {
        self._sync(true)
    }
}

/// Socket strategy when spawning [`Instance`]s using a local Unix domain socket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalUdsSocketStrategy {
    /// Randomly assign a socket from a temp file.
    Random,
    /// Randomly assign a socket from a temp file in the given parent directory.
    RandomIn(PathBuf),
    /// Use the given path as the socket location.
    Custom(PathBuf),
}

impl Default for LocalUdsSocketStrategy {
    fn default() -> Self {
        Self::Random
    }
}

impl LocalUdsSocketStrategy {
    /// Creates a random socket strategy.
    #[must_use]
    pub fn random() -> Self {
        Self::Random
    }

    /// Creates a random socket strategy in the given parent directory.
    pub fn random_in(path: impl Into<PathBuf>) -> Self {
        Self::RandomIn(path.into())
    }

    /// Creates a custom socket strategy for the given socket location.
    pub fn custom(path: impl Into<PathBuf>) -> Self {
        Self::Custom(path.into())
    }
}

fn temp_path_and_socket_from(
    socket_strategy: &LocalUdsSocketStrategy,
) -> Result<(Option<TempPath>, PathBuf)> {
    match socket_strategy {
        LocalUdsSocketStrategy::Random => {
            let temp_path = NamedTempFile::new()
                .map_err(LocalUdsInstanceError::TempSocket)?
                .into_temp_path();
            let socket = PathBuf::from(&temp_path);

            Ok((Some(temp_path), socket))
        }
        LocalUdsSocketStrategy::RandomIn(parent_path) => {
            let temp_path = NamedTempFile::new_in(parent_path)
                .map_err(LocalUdsInstanceError::TempSocket)?
                .into_temp_path();
            let socket = PathBuf::from(&temp_path);

            Ok((Some(temp_path), socket))
        }
        LocalUdsSocketStrategy::Custom(socket) => Ok((None, socket.clone())),
    }
}

async fn watch_task<Strm>(
    mut watch_progress: WatchStarted<Strm>,
    mut shutdown_rx: oneshot::Receiver<()>,
) where
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + Sync + 'static,
{
    loop {
        tokio::select! {
            // Got a shutdown message
            _ = Pin::new(&mut shutdown_rx) => {
                trace!("watch task received shutdown");
                if let Err(err) = watch_progress.stop().await {
                    warn!(error = ?err, "failed to cleanly close the watch session");
                }
                break;
            }
            // Got progress on the watch session
            result = watch_progress.next() => {
                match result {
                    // Got a ping, good news, proceed
                    Some(Ok(())) => {},
                    // An error occurred on the stream. We are going to treat this as catastrophic
                    // and end the watch.
                    Some(Err(err)) => {
                        warn!(error = ?err, "error on watch stream");
                        if let Err(err) = watch_progress.stop().await {
                            warn!(error = ?err, "failed to cleanly close the watch session");
                        }
                        break
                    }
                    // Stream is closed
                    None => {
                        trace!("watch stream has closed");
                        break
                    }
                }
            }
            // All other arms are closed, nothing left to do but return
            else => {
                trace!("returning from watch task with all select arms closed");
                break
            }
        }
    }
}
