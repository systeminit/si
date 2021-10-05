use derive_builder::Builder;
use std::{net::SocketAddr, path::PathBuf};

mod router;
pub mod telemetry;
pub mod uds;

pub use router::app;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    HTTPSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::HTTPSocket(SocketAddr::from(([0, 0, 0, 0], 8080)))
    }
}

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "false")]
    enable_ping: bool,

    #[builder(default = "true")]
    enable_resolver: bool,

    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Get a reference to the config's enable ping.
    pub fn enable_ping(&self) -> bool {
        self.enable_ping
    }

    /// Get a reference to the config's enable resolver.
    pub fn enable_resolver(&self) -> bool {
        self.enable_resolver
    }

    /// Get a reference to the config's incoming stream.
    pub fn incoming_stream(&self) -> &IncomingStream {
        &self.incoming_stream
    }
}
