use std::{
    io,
    path::{
        Path,
        PathBuf,
    },
    result,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use bollard::{
    Docker,
    container::{
        Config,
        CreateContainerOptions,
        RemoveContainerOptions,
        StartContainerOptions,
    },
    errors::Error,
    models::{
        HostConfig,
        Mount,
        MountTypeEnum,
    },
};
use cyclone_client::{
    Client,
    ClientConfig,
    ClientError,
    Connection,
    CycloneClient,
    Execution,
    LivenessStatus,
    PingExecution,
    ReadinessStatus,
    UdsClient,
    UnixStream,
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
use rand::{
    Rng,
    distributions::Alphanumeric,
    thread_rng,
};
use serde::{
    Deserialize,
    Serialize,
};
#[cfg(target_os = "linux")]
use si_firecracker::{
    errors::FirecrackerJailError,
    firecracker::FirecrackerJail,
};
use tempfile::{
    NamedTempFile,
    TempPath,
};
use thiserror::Error;
use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
    },
    process::{
        Child,
        Command,
    },
    sync::oneshot,
    time,
};
use tracing::{
    debug,
    trace,
};

use crate::instance::{
    Instance,
    Spec,
    SpecBuilder,
};

/// Error type for [`LocalUdsInstance`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum LocalUdsInstanceError {
    /// Spec builder error.
    #[error(transparent)]
    Builder(#[from] LocalUdsInstanceSpecBuilderError),
    /// Error when waiting for child process to shutdown.
    #[error(transparent)]
    ChildShutdown(#[from] ShutdownError),
    /// Failed to spawn a child process.
    #[error("failed to spawn cyclone child process: {0}")]
    ChildSpawn(#[source] io::Error),
    /// Cyclone client error.
    #[error(transparent)]
    Client(#[from] Box<ClientError>),
    /// Failed to build a container.
    #[error("failed to build a cyclone container: {0}")]
    ContainerBuild(#[source] Error),
    /// Failed to run a container.
    #[error("failed to spawn cyclone container: {0}")]
    ContainerRun(#[source] Error),
    /// Error when shutting down a container.
    #[error(transparent)]
    ContainerShutdown(#[from] Error),
    /// Docker api not found
    #[error("no docker api")]
    DockerAPINotFound,
    #[cfg(target_os = "linux")]
    /// Failed to firecracker jail.
    #[error("failed in working with a jail: {0}")]
    Firecracker(#[from] FirecrackerJailError),
    /// Failed to create firecracker-setup file.
    #[error("failed to create firecracker-setup file: {0}")]
    FirecrackerSetupCreate(#[source] io::Error),
    /// Failed to set permissions on the firecracker-setup file.
    #[error("failed to set permissions on the firecracker-setup file: {0}")]
    FirecrackerSetupPermissions(#[source] io::Error),
    /// Failed to run firecracker-setup file.
    #[error("failed to run firecracker-setup file: {0}")]
    FirecrackerSetupRun(String),
    /// Failed to write to firecracker-setup file.
    #[error("failed to write to firecracker-setup file: {0}")]
    FirecrackerSetupWrite(#[source] io::Error),
    /// Instance has exhausted its predefined request count.
    #[error("no remaining requests, cyclone server is considered unhealthy")]
    NoRemainingRequests,
    /// Failed to setup the host correctly.
    #[error("failed to setup host")]
    SetupFailed,
    /// Failed to create socket from temporary file.
    #[error("failed to create temp socket: {0}")]
    TempSocket(#[source] io::Error),
    /// Cyclone client `watch` endpoint error.
    #[error(transparent)]
    Watch(#[from] Box<WatchError>),
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

impl From<ClientError> for LocalUdsInstanceError {
    fn from(value: ClientError) -> Self {
        Box::new(value).into()
    }
}

impl From<WatchError> for LocalUdsInstanceError {
    fn from(value: WatchError) -> Self {
        Box::new(value).into()
    }
}

type Result<T> = result::Result<T, LocalUdsInstanceError>;

/// A local Cyclone [`Instance`], managed as a spawned child process, communicating over a Unix
/// domain socket ("Uds").
pub struct LocalUdsInstance {
    // The `TempPath` type is kept around as an [RAII
    // guard](https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html), that is,
    // when `LocalUdsInstance` is dropped, the temp file is marked for deletion.
    _temp_path: Option<TempPath>,
    client: UdsClient,
    limit_requests: Option<u32>,
    runtime: Box<dyn LocalInstanceRuntime>,
    watch_shutdown_tx: oneshot::Sender<()>,
}

// TODO(nick): make this more useful.
impl std::fmt::Debug for LocalUdsInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalUdsInstance").finish()
    }
}

#[async_trait]
impl Instance for LocalUdsInstance {
    type SpecBuilder = LocalUdsInstanceSpecBuilder;
    type Error = LocalUdsInstanceError;

    async fn terminate(&mut self) -> result::Result<(), Self::Error> {
        self.runtime.terminate().await
    }

    async fn ensure_healthy(&mut self) -> result::Result<(), Self::Error> {
        self.ensure_healthy_client().await?;

        Ok(())
    }
    fn id(&self) -> u32 {
        self.runtime.id()
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

    async fn prepare_execution<Request>(
        &mut self,
        request: CycloneRequest<Request>,
    ) -> result::Result<Execution<UnixStream, Request, Request::Response>, ClientError>
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

/// The [`Spec`] for [`LocalUdsInstance`]
#[derive(Builder, Clone, Debug, Default)]
pub struct LocalUdsInstanceSpec {
    /// Canonical path to the `cyclone` program.
    #[builder(try_setter, setter(into), default)]
    cyclone_cmd_path: CanonicalCommand,

    /// Canonical path to the language server program.
    #[builder(try_setter, setter(into), default)]
    lang_server_cmd_path: CanonicalCommand,

    /// Overrides the default function timeout for the language server program, in seconds.
    #[builder(default)]
    lang_server_function_timeout: Option<usize>,

    /// Socket strategy for a spawned Cyclone server.
    #[builder(default)]
    socket_strategy: LocalUdsSocketStrategy,

    /// Runtime strategy for a spawned Cyclone server.
    #[builder(default)]
    runtime_strategy: LocalUdsRuntimeStrategy,

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

    /// Enables the `remote_shell` execution endpoint for a spawned Cyclone server.
    #[builder(private, setter(name = "_remote_shell"), default = "false")]
    remote_shell: bool,

    /// Size of the pool to configure for the spec.
    #[builder(setter(into), default = "500")]
    pub pool_size: u32,

    /// Sets the timeout for connecting to firecracker
    #[builder(setter(into), default = "10")]
    connect_timeout: u64,

    /// Sets whether or not the firecracker setup scripts will be created.
    #[builder(default = "true")]
    create_firecracker_setup_scripts: bool,
}

#[async_trait]
impl Spec for LocalUdsInstanceSpec {
    type Instance = LocalUdsInstance;
    type Error = LocalUdsInstanceError;

    #[allow(unused_variables)]
    async fn clean(&self, id: u32) -> result::Result<(), Self::Error> {
        match self.runtime_strategy {
            LocalUdsRuntimeStrategy::LocalDocker => Ok(()),
            LocalUdsRuntimeStrategy::LocalProcess => Ok(()),
            #[cfg(target_os = "linux")]
            LocalUdsRuntimeStrategy::LocalFirecracker => LocalFirecrackerRuntime::clean(id).await,
        }
    }

    #[allow(unused_variables)]
    async fn prepare(&self, id: u32) -> result::Result<(), Self::Error> {
        match self.runtime_strategy {
            LocalUdsRuntimeStrategy::LocalDocker => Ok(()),
            LocalUdsRuntimeStrategy::LocalProcess => Ok(()),
            #[cfg(target_os = "linux")]
            LocalUdsRuntimeStrategy::LocalFirecracker => LocalFirecrackerRuntime::prepare(id).await,
        }
    }

    #[allow(unused_variables)]
    async fn setup(&mut self) -> result::Result<(), Self::Error> {
        match self.runtime_strategy {
            LocalUdsRuntimeStrategy::LocalDocker => Ok(()),
            LocalUdsRuntimeStrategy::LocalProcess => Ok(()),
            #[cfg(target_os = "linux")]
            LocalUdsRuntimeStrategy::LocalFirecracker => {
                LocalFirecrackerRuntime::setup_firecracker(self).await
            }
        }
    }

    #[allow(unused_assignments, unused_mut)]
    async fn spawn(&self, id: u32) -> result::Result<Self::Instance, Self::Error> {
        let (temp_path, socket) = temp_path_and_socket_from(&self.socket_strategy)?;
        let mut runtime = runtime_instance_from_spec(self, &socket, id).await?;

        runtime.spawn().await?;
        //TODO(scott): Firecracker requires the client to add a special connection detail. We
        //should find a better way to handle this.
        let mut firecracker_connect = false;
        #[cfg(target_os = "linux")]
        {
            firecracker_connect = matches!(
                self.runtime_strategy,
                LocalUdsRuntimeStrategy::LocalFirecracker
            );
        }

        let config = ClientConfig {
            connect_timeout: Duration::from_millis(self.connect_timeout),
            firecracker_connect,
            ..Default::default()
        };
        let mut client = Client::uds(runtime.socket(), Arc::new(config))?;

        // Establish the client watch session. As the process may be booting, we will retry for a
        // period before giving up and assuming that the server instance has failed.
        let watch = {
            let mut retries = 300;
            loop {
                match client.watch().await {
                    Ok(watch) => {
                        break watch;
                    }
                    Err(err) => err,
                };
                if retries < 1 {
                    runtime.terminate().await?;
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
            runtime,
            watch_shutdown_tx,
        })
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

    /// Enables the `resolver` execution endpoint for a spawned Cyclone server.
    pub fn resolver(&mut self) -> &mut Self {
        self._resolver(true)
    }

    /// Enables the `action` execution endpoint for a spawned Cyclone server.
    pub fn action(&mut self) -> &mut Self {
        self._action(true)
    }

    /// Enables the `remote_shell` execution endpoint for a spawned Cyclone server.
    pub fn remote_shell(&mut self) -> &mut Self {
        self._remote_shell(true)
    }

    /// Enables all available endpoints for a spawned Cyclone server
    pub fn all_endpoints(&mut self) -> &mut Self {
        self.action().resolver().remote_shell()
    }
}

/// Socket strategy when spawning [`Instance`]s using a local Unix domain socket.
#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum LocalUdsSocketStrategy {
    /// Use the given path as the socket location.
    Custom(PathBuf),
    /// Randomly assign a socket from a temp file.
    Random,
    /// Randomly assign a socket from a temp file in the given parent directory.
    RandomIn(PathBuf),
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
            let temp_path = NamedTempFile::with_prefix("cyclone")
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

#[remain::sorted]
/// Runtime strategy when spawning [`Instance`]s.
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum LocalUdsRuntimeStrategy {
    /// Run Docker containers on the local machine
    LocalDocker,
    #[cfg(target_os = "linux")]
    /// Run processes on firecracker
    LocalFirecracker,
    /// Run processes on the local machine
    LocalProcess,
}

impl Default for LocalUdsRuntimeStrategy {
    fn default() -> Self {
        // firecracker-setup: change LocalProcess to LocalFirecracker
        Self::LocalProcess
    }
}

#[async_trait]
pub trait LocalInstanceRuntime: Send + Sync {
    fn id(&self) -> u32;
    fn socket(&mut self) -> PathBuf;
    async fn spawn(&mut self) -> result::Result<(), LocalUdsInstanceError>;
    async fn terminate(&mut self) -> result::Result<(), LocalUdsInstanceError>;
}

#[derive(Debug)]
struct LocalProcessRuntime {
    cmd: Command,
    child: Option<Child>,
    socket: PathBuf,
}

impl LocalProcessRuntime {
    async fn build(
        socket: &PathBuf,
        spec: LocalUdsInstanceSpec,
    ) -> Result<Box<dyn LocalInstanceRuntime>> {
        let mut cmd = Command::new(&spec.cyclone_cmd_path);
        cmd.arg("--bind-uds")
            .arg(socket)
            .arg("--lang-server")
            .arg(&spec.lang_server_cmd_path)
            .arg("--enable-watch");
        if let Some(timeout) = spec.lang_server_function_timeout {
            cmd.arg("--timeout").arg(timeout.to_string());
        }
        if let Some(limit_requests) = spec.limit_requests {
            cmd.arg("--limit-requests").arg(limit_requests.to_string());
        }
        if let Some(timeout) = spec.watch_timeout {
            cmd.arg("--watch-timeout")
                .arg(timeout.as_secs().to_string());
        }
        if spec.ping {
            cmd.arg("--enable-ping");
        }
        if spec.resolver {
            cmd.arg("--enable-resolver");
        }
        if spec.action {
            cmd.arg("--enable-action-run");
        }
        if spec.remote_shell {
            cmd.arg("--enable-remote-shell");
        }

        Ok(Box::new(LocalProcessRuntime {
            cmd,
            child: None,
            socket: socket.to_path_buf(),
        }))
    }
}

#[async_trait]
impl LocalInstanceRuntime for LocalProcessRuntime {
    fn id(&self) -> u32 {
        0
    }
    fn socket(&mut self) -> PathBuf {
        self.socket.to_path_buf()
    }

    async fn spawn(&mut self) -> result::Result<(), LocalUdsInstanceError> {
        self.child = Some(
            self.cmd
                .spawn()
                .map_err(LocalUdsInstanceError::ChildSpawn)?,
        );
        Ok(())
    }
    async fn terminate(&mut self) -> result::Result<(), LocalUdsInstanceError> {
        match self.child.as_mut() {
            Some(c) => {
                process::child_shutdown(c, Some(process::Signal::SIGTERM), None).await?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

#[derive(Debug)]
struct LocalDockerRuntime {
    container_id: String,
    docker: Docker,
    socket: PathBuf,
}

impl LocalDockerRuntime {
    async fn build(
        socket: &Path,
        spec: LocalUdsInstanceSpec,
    ) -> Result<Box<dyn LocalInstanceRuntime>> {
        let mut cmd = vec![
            String::from("--bind-uds"),
            socket.to_string_lossy().to_string(),
            String::from("--lang-server"),
            String::from("/usr/local/bin/lang-js"),
            String::from("--enable-watch"),
        ];
        if let Some(limit_requests) = spec.limit_requests {
            cmd.push(String::from("--limit-requests"));
            cmd.push(limit_requests.to_string())
        }
        if let Some(timeout) = spec.watch_timeout {
            cmd.push(String::from("--watch-timeout"));
            cmd.push(timeout.as_secs().to_string());
        }
        if spec.ping {
            cmd.push(String::from("--enable-ping"));
        }
        if spec.resolver {
            cmd.push(String::from("--enable-resolver"));
        }
        if spec.action {
            cmd.push(String::from("--enable-action-run"));
        }
        if spec.remote_shell {
            cmd.push(String::from("--enable-remote-shell"));
        }

        let docker = Docker::connect_with_local_defaults()?;

        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let socket_dir = socket
            .parent()
            .expect("socket path not available")
            .to_str()
            .expect("unable to unpack path");
        let mounts = vec![Mount {
            source: Some(String::from(socket_dir)),
            target: Some(String::from(socket_dir)),
            typ: Some(MountTypeEnum::BIND),
            ..Default::default()
        }];

        let container_id = docker
            .create_container(
                Some(CreateContainerOptions {
                    name: format!("cyclone-container-{rand_string}"),
                    platform: Some(String::from("linux/amd64")),
                }),
                Config {
                    image: Some(String::from("systeminit/cyclone:stable")),
                    cmd: Some(cmd),
                    host_config: Some(HostConfig {
                        mounts: Some(mounts),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?
            .id;

        Ok(Box::new(LocalDockerRuntime {
            container_id,
            docker,
            socket: socket.to_path_buf(),
        }))
    }
}

#[async_trait]
impl LocalInstanceRuntime for LocalDockerRuntime {
    fn id(&self) -> u32 {
        0
    }
    fn socket(&mut self) -> PathBuf {
        self.socket.to_path_buf()
    }

    async fn spawn(&mut self) -> result::Result<(), LocalUdsInstanceError> {
        self.docker
            .start_container(
                &self.container_id.clone(),
                None::<StartContainerOptions<String>>,
            )
            .await?;
        Ok(())
    }

    async fn terminate(&mut self) -> result::Result<(), LocalUdsInstanceError> {
        self.docker
            .remove_container(
                &self.container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
#[cfg(target_os = "linux")]
struct LocalFirecrackerRuntime {
    jail: FirecrackerJail,
    vm_id: u32,
}

#[cfg(target_os = "linux")]
impl LocalFirecrackerRuntime {
    async fn build(_spec: LocalUdsInstanceSpec, id: u32) -> Result<Box<dyn LocalInstanceRuntime>> {
        let jail = FirecrackerJail::build(id).await?;
        Ok(Box::new(LocalFirecrackerRuntime { jail, vm_id: id }))
    }
}

#[async_trait]
#[cfg(target_os = "linux")]
impl LocalInstanceRuntime for LocalFirecrackerRuntime {
    fn id(&self) -> u32 {
        self.vm_id
    }
    fn socket(&mut self) -> PathBuf {
        self.jail.socket()
    }

    async fn spawn(&mut self) -> Result<()> {
        Ok(self.jail.spawn().await?)
    }

    async fn terminate(&mut self) -> Result<()> {
        Ok(self.jail.terminate().await?)
    }
}

#[cfg(target_os = "linux")]
impl LocalFirecrackerRuntime {
    async fn clean(id: u32) -> Result<()> {
        Ok(FirecrackerJail::clean(id).await?)
    }

    async fn prepare(id: u32) -> Result<()> {
        Ok(FirecrackerJail::prepare(id).await?)
    }

    async fn setup_firecracker(spec: &LocalUdsInstanceSpec) -> Result<()> {
        Ok(FirecrackerJail::setup(spec.pool_size, spec.create_firecracker_setup_scripts).await?)
    }
}

#[allow(unused_variables)]
async fn runtime_instance_from_spec(
    spec: &LocalUdsInstanceSpec,
    socket: &PathBuf,
    id: u32,
) -> Result<Box<dyn LocalInstanceRuntime>> {
    match spec.runtime_strategy {
        LocalUdsRuntimeStrategy::LocalProcess => {
            LocalProcessRuntime::build(socket, spec.clone()).await
        }
        LocalUdsRuntimeStrategy::LocalDocker => {
            LocalDockerRuntime::build(socket, spec.clone()).await
        }
        #[cfg(target_os = "linux")]
        LocalUdsRuntimeStrategy::LocalFirecracker => {
            LocalFirecrackerRuntime::build(spec.clone(), id).await
        }
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
                    trace!(error = ?err, "failed to cleanly close the watch session");
                }
                break;
            }
            // Got progress on the watch session
            result = watch_progress.next() => {
                match result {
                    // Got a ping, good news, proceed
                    Some(Ok(())) => {

                    },
                    // An error occurred on the stream. We are going to treat this as catastrophic
                    // and end the watch.
                    Some(Err(err)) => {
                        debug!(error = ?err, "error on watch stream");
                        if let Err(err) = watch_progress.stop().await {
                            debug!(error = ?err, "failed to cleanly close the watch session");
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
