use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};

use derive_builder::Builder;
use si_settings::error::SettingsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Builder(#[from] ConfigBuilderError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    nats_url: String,

    #[builder(default = "CycloneStream::default()")]
    cyclone_stream: CycloneStream,
}

impl Config {
    /// Constructs a builder for creating a Config
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Gets a reference to the config's nats url.
    pub fn nats_url(&self) -> &str {
        self.nats_url.as_str()
    }

    /// Gets a reference to the config's cyclone stream.
    pub fn cyclone_stream(&self) -> &CycloneStream {
        &self.cyclone_stream
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CycloneStream {
    HttpSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for CycloneStream {
    fn default() -> Self {
        Self::HttpSocket(SocketAddr::from(([0, 0, 0, 0], 5157)))
    }
}

impl CycloneStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .into_iter()
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HttpSocket(socket_addr))
    }
    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}
