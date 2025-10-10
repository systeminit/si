use std::{
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
    },
    result,
    time::Duration,
};

use async_trait::async_trait;
use cyclone_client::{
    Client,
    ClientError,
    Connection,
    CycloneClient,
    Execution,
    HttpClient,
    LivenessStatus,
    PingExecution,
    ReadinessStatus,
    Watch,
    WatchError,
    WatchStarted,
    new_unstarted_execution,
};
use cyclone_core::{
    CanonicalCommand,
    CycloneRequest,
    CycloneRequestable,
    process::{
        self,
        ShutdownError,
    },
};
use derive_builder::Builder;
use futures::StreamExt;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use tokio::{
    io::{
        self,
        AsyncRead,
        AsyncWrite,
    },
    net::{
        TcpListener,
        TcpStream,
    },
    process::{
        Child,
        Command,
    },
    sync::oneshot,
    time::{
        self,
    },
};
use tracing::{
    debug,
    trace,
    warn,
};

use crate::instance::{
    Instance,
    Spec,
    SpecBuilder,
};

/// Error type for [`LocalHttpInstance`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum LocalHttpInstanceError {
    /// Spec builder error.
    #[error("builder error: {0}")]
    Builder(#[from] LocalHttpInstanceSpecBuilderError),
    /// Error when waiting for child process to shutdown.
    #[error("child shutdown error: {0}")]
    ChildShutdown(#[from] ShutdownError),
    /// Failed to spawn a child process.
    #[error("failed to spawn cyclone child process")]
    ChildSpawn(#[source] io::Error),
    /// Cyclone client error.
    #[error("client error: {0}")]
    Client(#[from] ClientError),
    /// Instance has exhausted its predefined request count.
    #[error("no remaining requests, cyclone server is considered unhealthy")]
    NoRemainingRequests,
    /// Error when binding local socket.
    #[error("error when binding local socket")]
    SocketBind(#[source] io::Error),
    /// Cyclone client `watch` endpoint error.
    #[error("watch error: {0}")]
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

type Result<T> = result::Result<T, LocalHttpInstanceError>;

/// A local Cyclone [`Instance`], managed as a spawned child process, communicating over HTTP.
#[derive(Debug)]
pub struct LocalHttpInstance {
    client: HttpClient,
    limit_requests: Option<u32>,
    child: Child,
    watch_shutdown_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Instance for LocalHttpInstance {
    type SpecBuilder = LocalHttpInstanceSpecBuilder;
    type Error = LocalHttpInstanceError;

    async fn terminate(&mut self) -> result::Result<(), Self::Error> {
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

    fn id(&self) -> u32 {
        0
    }
}

#[async_trait]
impl CycloneClient<TcpStream> for LocalHttpInstance {
    async fn watch(&mut self) -> result::Result<Watch<TcpStream>, ClientError> {
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

    async fn execute_ping(&mut self) -> result::Result<PingExecution<TcpStream>, ClientError> {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_ping().await;
        self.count_request();

        result
    }

    async fn prepare_execution<Request>(
        &mut self,
        request: CycloneRequest<Request>,
    ) -> result::Result<Execution<TcpStream, Request, Request::Response>, ClientError>
    where
        Request: CycloneRequestable + Send + Sync,
    {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;
        let stream = self
            .client
            .websocket_stream(request.websocket_path())
            .await?;
        let result = new_unstarted_execution(stream, request);
        self.count_request();

        Ok(result)
    }
}

impl LocalHttpInstance {
    async fn ensure_healthy_client(&mut self) -> Result<()> {
        if !self.is_watch_shutdown_open() {
            return Err(LocalHttpInstanceError::WatchShutDown);
        }
        if !self.has_remaining_requests() {
            return Err(LocalHttpInstanceError::NoRemainingRequests);
        }

        Ok(())
    }

    fn has_remaining_requests(&self) -> bool {
        match self.limit_requests {
            Some(0) => false,
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

/// The [`Spec`] for [`LocalHttpInstance`]
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
pub struct LocalHttpInstanceSpec {
    /// Canonical path to the `cyclone` program.
    #[builder(try_setter, setter(into))]
    cyclone_cmd_path: CanonicalCommand,

    /// Canonical path to the language server program.
    #[builder(try_setter, setter(into))]
    lang_server_cmd_path: CanonicalCommand,

    /// Overrides the default function timeout for the language server program, in seconds.
    #[builder(default)]
    lang_server_function_timeout: Option<usize>,

    /// Socket strategy for a spawned Cyclone server.
    #[builder(default)]
    socket_strategy: LocalHttpSocketStrategy,

    /// Sets the watch timeout value for a spawned Cyclone server.
    #[builder(setter(into, strip_option), default)]
    watch_timeout: Option<Duration>,

    /// Sets the limit requests strategy for a spawned Cyclone server.
    #[builder(setter(into), default = "Some(1)")]
    limit_requests: Option<u32>,

    /// Enables the `ping` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_ping"), default = "false")]
    ping: bool,

    /// Enables the `resolver` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_resolver"), default = "false")]
    resolver: bool,

    /// Enables the `action` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_action"), default = "false")]
    action: bool,
}

#[async_trait]
impl Spec for LocalHttpInstanceSpec {
    type Instance = LocalHttpInstance;
    type Error = LocalHttpInstanceError;

    async fn clean(&self, _id: u32) -> result::Result<(), Self::Error> {
        Ok(())
    }
    async fn prepare(&self, _id: u32) -> result::Result<(), Self::Error> {
        Ok(())
    }
    async fn setup(&mut self) -> result::Result<(), Self::Error> {
        Ok(())
    }
    async fn spawn(&self, _id: u32) -> result::Result<Self::Instance, Self::Error> {
        let socket_addr = socket_addr_from(&self.socket_strategy).await?;
        let mut cmd = self.build_command(&socket_addr);

        debug!("spawning child process; cmd={:?}", &cmd);
        let child = cmd.spawn().map_err(Self::Error::ChildSpawn)?;

        let mut client = Client::http(socket_addr)?;

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
            client,
            limit_requests: self.limit_requests,
            child,
            watch_shutdown_tx,
        })
    }
}

impl LocalHttpInstanceSpec {
    fn build_command(&self, socket: &SocketAddr) -> Command {
        let mut cmd = Command::new(&self.cyclone_cmd_path);
        cmd.arg("--bind-addr")
            .arg(socket.to_string())
            .arg("--lang-server")
            .arg(&self.lang_server_cmd_path)
            .arg("--enable-watch");
        if let Some(timeout) = self.lang_server_function_timeout {
            cmd.arg("--timeout").arg(timeout.to_string());
        }
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
        if self.resolver {
            cmd.arg("--enable-resolver");
        }
        if self.action {
            cmd.arg("--enable-action-run");
        }

        cmd
    }
}

impl SpecBuilder for LocalHttpInstanceSpecBuilder {
    type Spec = LocalHttpInstanceSpec;
    type Error = LocalHttpInstanceError;

    fn build(&self) -> result::Result<Self::Spec, Self::Error> {
        self.build().map_err(Into::into)
    }
}

impl LocalHttpInstanceSpecBuilder {
    /// Sets the limit requests strategy to `1` for a spawned Cyclone server.
    pub fn oneshot(&mut self) -> &mut Self {
        self.limit_requests(Some(1))
    }

    /// Enables the `ping` execution endpoint for a spawned Cyclone server.
    pub fn ping(&mut self) -> &mut Self {
        self._ping(true)
    }

    /// Enables the `resolver` execution endpoint for a spawned Cyclone server.
    pub fn resolver(&mut self) -> &mut Self {
        self._resolver(true)
    }

    /// Enables the `action` execution endpoint for a spawned Cyclone server.
    pub fn action(&mut self) -> &mut Self {
        self._action(true)
    }

    /// Enables all available endpoints for a spawned Cyclone server
    pub fn all_endpoints(&mut self) -> &mut Self {
        self.action().resolver()
    }
}

/// Socket strategy when spawning [`Instance`]s using a TCP socket.
#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum LocalHttpSocketStrategy {
    /// Use the given port.
    Custom(u16),
    /// Randomly assign a port.
    Random,
}

impl Default for LocalHttpSocketStrategy {
    fn default() -> Self {
        Self::Random
    }
}

impl LocalHttpSocketStrategy {
    /// Creates a random socket strategy.
    #[must_use]
    pub fn random() -> Self {
        Self::Random
    }

    /// Creates a custom port strategy.
    pub fn custom(port: impl Into<u16>) -> Self {
        Self::Custom(port.into())
    }
}

async fn socket_addr_from(socket_strategy: &LocalHttpSocketStrategy) -> Result<SocketAddr> {
    match socket_strategy {
        LocalHttpSocketStrategy::Random => {
            // NOTE(fnichol): we're asking the kernel to give us a currently unassigned port, then
            // immediately attempting to give that port to another spawned process. You could
            // probably think of scenarios where the spawned program might not be able to bind to
            // this, right? However, for the moment we should be reasonably okay--this pooling code
            // would attempt to spawn another Cyclone instance if one couldn't be booted up
            // correctly, so a bit of a retry is built in here. WHAT COULD GO WRONG!?
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(LocalHttpInstanceError::SocketBind)?;
            listener
                .local_addr()
                .map_err(LocalHttpInstanceError::SocketBind)
        }
        LocalHttpSocketStrategy::Custom(port) => Ok(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            *port,
        ))),
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
                    warn!(si.error.message = ?err, "failed to cleanly close the watch session");
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
                        warn!(si.error.message = ?err, "error on watch stream");
                        if let Err(err) = watch_progress.stop().await {
                            warn!(si.error.message = ?err, "failed to cleanly close the watch session");
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
