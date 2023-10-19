use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    result,
    time::Duration,
};

use async_trait::async_trait;
use cyclone_client::{
    Client, ClientError, Connection, CycloneClient, Execution, HttpClient, LivenessStatus,
    PingExecution, ReadinessStatus, Watch, WatchError, WatchStarted,
};
use cyclone_core::{
    process::{self, ShutdownError},
    ActionRunRequest, ActionRunResultSuccess, CanonicalCommand, ReconciliationRequest,
    ReconciliationResultSuccess, ResolverFunctionRequest, ResolverFunctionResultSuccess,
    SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess, ValidationRequest,
    ValidationResultSuccess,
};
use derive_builder::Builder;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    process::{Child, Command},
    sync::oneshot,
    time,
};
use tracing::{debug, trace, warn};

use crate::instance::{Instance, Spec, SpecBuilder};

use rand::Rng; // Import the rand crate for random port generation
use docker::container::{CreateContainer, CreateContainerOptions, Logs, RemoveContainer, StartContainer, Stdio};
use docker::Docker;
use docker::DockerError;
use docker::image::{BuildInfo, Config};
use docker::image::Image;

/// Error type for [`LocalDockerInstance`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum LocalDockerInstanceError {
    /// Spec builder error.
    #[error(transparent)]
    Builder(#[from] LocalDockerInstanceSpecBuilderError),
    /// Error when waiting for child process to shutdown.
    #[error(transparent)]
    ChildShutdown(#[from] ShutdownError),
    /// Failed to spawn a Docker container.
    #[error("failed to create Cyclone Docker container")]
    DockerContainerCreation(#[source] DockerError),
    /// Failed to start the Docker container.
    #[error("failed to start Cyclone Docker container")]
    DockerContainerStart(#[source] DockerError),
    /// Failed to remove the Docker container.
    #[error("failed to remove Cyclone Docker container")]
    DockerContainerRemoval(#[source] DockerError),
    /// Cyclone client error.
    #[error(transparent)]
    Client(#[from] ClientError),
    /// Instance has exhausted its predefined request count.
    #[error("no remaining requests, cyclone server is considered unhealthy")]
    NoRemainingRequests,
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

type Result<T> = result::Result<T, LocalDockerInstanceError>;

/// A local Cyclone [`Instance`], managed as a Docker container, communicating over HTTP.
#[derive(Debug)]
pub struct LocalDockerInstance {
    client: HttpClient,
    limit_requests: Option<u32>,
    docker: Docker,
    container: Container,
    watch_shutdown_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Instance for LocalDockerInstance {
    type SpecBuilder = LocalDockerInstanceSpecBuilder;
    type Error = LocalDockerInstanceError;

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
impl CycloneClient<TcpStream> for LocalDockerInstance {
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

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> result::Result<
        Execution<TcpStream, ResolverFunctionRequest, ResolverFunctionResultSuccess>,
        ClientError,
    > {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_resolver(request).await;
        self.count_request();

        result
    }

    async fn execute_validation(
        &mut self,
        request: ValidationRequest,
    ) -> result::Result<Execution<TcpStream, ValidationRequest, ValidationResultSuccess>, ClientError>
    {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_validation(request).await;
        self.count_request();

        result
    }

    async fn execute_action_run(
        &mut self,
        request: ActionRunRequest,
    ) -> result::Result<Execution<TcpStream, ActionRunRequest, ActionRunResultSuccess>, ClientError>
    {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_action_run(request).await;
        self.count_request();

        result
    }

    async fn execute_reconciliation(
        &mut self,
        request: ReconciliationRequest,
    ) -> result::Result<
        Execution<TcpStream, ReconciliationRequest, ReconciliationResultSuccess>,
        ClientError,
    > {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_reconciliation(request).await;
        self.count_request();

        result
    }

    async fn execute_schema_variant_definition(
        &mut self,
        request: SchemaVariantDefinitionRequest,
    ) -> result::Result<
        Execution<TcpStream, SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess>,
        ClientError,
    > {
        self.ensure_healthy_client()
            .await
            .map_err(ClientError::unhealthy)?;

        let result = self.client.execute_schema_variant_definition(request).await;
        self.count_request();

        result
    }
}

impl LocalDockerInstance {
    async fn ensure_healthy_client(&mut self) -> Result<()> {
        if !self.is_watch_shutdown_open() {
            return Err(LocalDockerInstanceError::WatchShutDown);
        }
        if !self.has_remaining_requests() {
            return Err(LocalDockerInstanceError::NoRemainingRequests);
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

/// The [`Spec`] for [`LocalDockerInstance`]
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
pub struct LocalDockerInstanceSpec {
    /// Canonical path to the language server program.
    #[builder(try_setter, setter(into))]
    lang_server_cmd_path: CanonicalCommand,

    /// Canonical path to Cyclone's secret key file.
    #[builder(setter(into))]
    cyclone_decryption_key_path: String,

    /// Socket strategy for a spawned Cyclone server.
    #[builder(default)]
    socket_strategy: LocalDockerSocketStrategy,

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
    #[builder(private, setter(name = "_action"), default = "false"]
    action: bool,

    /// Randomly assigned port between 20000 and 30000
    #[builder(setter(strip_option), default)]
    random_port: Option<u16>,
}

#[async_trait]
impl Spec for LocalDockerInstanceSpec {
    type Instance = LocalDockerInstance;
    type Error = LocalDockerInstanceError;

    async fn spawn(&self) -> result::Result<Self::Instance, Self::Error> {
        // Generate a random port if not specified
        let random_port = self.random_port.unwrap_or_else(|| {
            rand::thread_rng().gen_range(20000..30001)
        });

        // Create a Docker client
        let docker = Docker::connect_with_defaults()?;

        let dockerfile_content = format!(
            r#"FROM debian:bullseye-slim
            WORKDIR /
            COPY {} /entrypoint.sh
            COPY {} /cyclone-decryption-key.pem
            CMD ["/entrypoint.sh"]
            EXPOSE {}   # Expose the random port
            "#,
            self.lang_server_cmd_path, self.cyclone_decryption_key_path, random_port
        );

        // Build the custom Docker image
        let build_info = docker.build_image(
            CreateContainerOptions {
                name: "cyclone-builder",
            },
            BuildInfo::new(
                dockerfile_content.as_bytes(),
                None,
                None,
                None,
                "Dockerfile",
                true,
            ),
        )?;

        // Start the Docker container from the custom image
        let container = docker.create_container(
            Some(CreateContainerOptions {
                name: "cyclone-container",
            }),
            Config {
                image: build_info.image.id,
                cmd: Some(vec![]),
                host_config: Some(docker::container::HostConfig {
                    port_bindings: Some(vec![(
                        random_port.into(),
                        Some(vec![docker::container::PortBinding {
                            host_ip: "0.0.0.0".to_string(),
                            host_port: format!("{}", random_port),
                        }]),
                    )]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )?;
        docker.start_container(&container, None)?;

        // Wait for the container to be healthy
        wait_for_container_health(&docker, &container)?;

        let mut client = Client::http(format!("localhost:{}", random_port))?; // Use the random port for client

        // Establish the client watch session. As the process may be booting, we will retry for a period before giving up and assuming that the server instance has failed.
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
        // Establish that we have received our first watch ping, which should happen immediately after establishing a watch session
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
            docker,
            container,
            watch_shutdown_tx,
        })
    }
}

impl LocalDockerInstanceSpec {
    fn build_command(&self, socket: &SocketAddr) -> Command {
        let mut cmd = Command::new(&self.cyclone_cmd_path);
        cmd.arg("--bind-addr")
            .arg(socket.to_string())
            .arg("--decryption-key")
            .arg(&self.cyclone_decryption_key_path)
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
        if self.resolver {
            cmd.arg("--enable-resolver");
        }
        if self.action {
            cmd.arg("--enable-action-run");
        }

        cmd
    }
}

impl SpecBuilder for LocalDockerInstanceSpecBuilder {
    type Spec = LocalDockerInstanceSpec;
    type Error = LocalDockerInstanceError;

    fn build(&self) -> result::Result<Self::Spec, Self::Error> {
        self.build().map_err(Into::into)
    }
}

impl LocalDockerInstanceSpecBuilder {
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
pub enum LocalDockerSocketStrategy {
    /// Use the given port.
    Custom(u16),
    /// Randomly assign a port.
    Random,
}

impl Default for LocalDockerSocketStrategy {
    fn default() -> Self {
        Self::Random
    }
}

impl LocalDockerSocketStrategy {
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

async fn socket_addr_from(socket_strategy: &LocalDockerSocketStrategy) -> Result<SocketAddr> {
    match socket_strategy {
        LocalDockerSocketStrategy::Random => {
            // NOTE: We are now using the random port assigned to the Docker container.
            // No need to generate a random port here.
            Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), random_port)))
        }
        LocalDockerSocketStrategy::Custom(port) => Ok(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            *port,
        )),
    }
}

async fn wait_for_container_health(docker: &Docker, container: &Container) -> Result<()> {
    let mut health_check_failed = false;
    let mut retries = 60; // Adjust the number of retries as needed
    while retries > 0 {
        let container_info = docker.inspect_container(container)?;
        if let Some(health) = container_info.state.health {
            if health.status != "healthy" {
                health_check_failed = true;
            } else {
                health_check_failed = false;
                break;
            }
        }
        retries -= 1;
        time::sleep(Duration::from_secs(1)).await;
    }

    if health_check_failed {
        return Err(LocalDockerInstanceError::DockerContainerStart(
            DockerError::HealthCheckFailed,
        ));
    }

    Ok(())
}