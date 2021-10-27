use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};

use derive_builder::Builder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,
}

impl Config {
    /// Constructs a builder for creating a Config
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Gets a reference to the config's incoming stream.
    pub fn incoming_stream(&self) -> &IncomingStream {
        &self.incoming_stream
    }
}

impl ConfigBuilder {
    pub fn http_socket(&mut self, socket_addrs: impl ToSocketAddrs) -> Result<&mut Self> {
        Ok(self.incoming_stream(IncomingStream::http_socket(socket_addrs)?))
    }

    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    HTTPSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::HTTPSocket(SocketAddr::from(([0, 0, 0, 0], 5156)))
    }
}

impl IncomingStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .into_iter()
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HTTPSocket(socket_addr))
    }

    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}
