use std::{
    net::{
        SocketAddr,
        ToSocketAddrs,
    },
    path::{
        Path,
        PathBuf,
    },
    time::Duration,
};

use derive_builder::Builder;
use si_std::{
    CanonicalFile,
    CanonicalFileError,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default)]
    watch: Option<Duration>,

    #[builder(default = "false")]
    enable_ping: bool,

    #[builder(default = "true")]
    enable_resolver: bool,

    #[builder(default = "true")]
    enable_action_run: bool,

    #[builder(default = "true")]
    enable_validation: bool,

    #[builder(default = "true")]
    enable_schema_variant_definition: bool,

    #[builder(default = "true")]
    enable_management: bool,

    #[builder(default = "true")]
    enable_debug: bool,

    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,

    #[builder(try_setter, setter(into))]
    lang_server_path: CanonicalFile,

    #[builder(default)]
    lang_server_function_timeout: Option<usize>,

    #[builder(default)]
    lang_server_process_timeout: Option<u64>,

    #[builder(setter(into), default)]
    limit_requests: Option<u32>,

    #[builder(setter(into), default = "false")]
    enable_forwarder: bool,

    #[builder(setter(into), default = "false")]
    enable_process_gatherer: bool,
}

impl Config {
    /// Constructs a builder for creating a Config
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Gets a reference to the config's watch.
    #[must_use]
    pub fn watch(&self) -> Option<Duration> {
        self.watch
    }

    /// Gets a reference to the config's enable ping.
    #[must_use]
    pub fn enable_ping(&self) -> bool {
        self.enable_ping
    }

    /// Gets a reference to the config's enable resolver.
    #[must_use]
    pub fn enable_resolver(&self) -> bool {
        self.enable_resolver
    }

    /// Gets a reference to the config's enable action run.
    #[must_use]
    pub fn enable_action_run(&self) -> bool {
        self.enable_action_run
    }

    /// Gets a reference to the config's enable validation
    #[must_use]
    pub fn enable_validation(&self) -> bool {
        self.enable_validation
    }

    /// Gets the config's enable schema_variant_definition
    #[must_use]
    pub fn enable_schema_variant_definition(&self) -> bool {
        self.enable_schema_variant_definition
    }

    /// Gets the config's enable schema_variant_definition
    #[must_use]
    pub fn enable_management(&self) -> bool {
        self.enable_management
    }

    /// Gets the config's enable debug
    #[must_use]
    pub fn enable_debug(&self) -> bool {
        self.enable_debug
    }

    /// Gets a reference to the config's incoming stream.
    #[must_use]
    pub fn incoming_stream(&self) -> &IncomingStream {
        &self.incoming_stream
    }

    /// Gets a reference to the config's lang server path.
    #[must_use]
    pub fn lang_server_path(&self) -> &Path {
        self.lang_server_path.as_path()
    }

    /// Gets a reference to the config's lang server function timeout optional override.
    #[must_use]
    pub fn lang_server_function_timeout(&self) -> Option<usize> {
        self.lang_server_function_timeout
    }

    /// Gets a reference to the config's lang server process timeout optional override.
    #[must_use]
    pub fn lang_server_process_timeout(&self) -> Option<u64> {
        self.lang_server_process_timeout
    }

    /// Gets a reference to the config's limit requests.
    #[must_use]
    pub fn limit_requests(&self) -> Option<u32> {
        self.limit_requests
    }

    /// Gets a reference to the config's enable forwarder.
    #[must_use]
    pub fn enable_forwarder(&self) -> bool {
        self.enable_forwarder
    }

    /// Gets a reference to the config's enable process gatherer.
    #[must_use]
    pub fn enable_process_gatherer(&self) -> bool {
        self.enable_process_gatherer
    }
}

impl ConfigBuilder {
    pub fn http_socket(&mut self, socket_addrs: impl ToSocketAddrs) -> Result<&mut Self> {
        Ok(self.incoming_stream(IncomingStream::http_socket(socket_addrs)?))
    }

    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }

    #[cfg(target_os = "linux")]
    pub fn vsock_socket(&mut self, addr: tokio_vsock::VsockAddr) -> &mut Self {
        self.incoming_stream(IncomingStream::vsock_socket(addr))
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    HTTPSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
    #[cfg(target_os = "linux")]
    VsockSocket(tokio_vsock::VsockAddr),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::HTTPSocket(SocketAddr::from(([0, 0, 0, 0], 5157)))
    }
}

impl IncomingStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HTTPSocket(socket_addr))
    }

    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }

    #[cfg(target_os = "linux")]
    pub fn vsock_socket(addr: tokio_vsock::VsockAddr) -> Self {
        Self::VsockSocket(addr)
    }
}
