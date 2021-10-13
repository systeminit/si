use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use axum::routing::{BoxRoute, IntoMakeService};
use hyper::server::conn::AddrIncoming;
use thiserror::Error;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("UDS incoming stream error")]
    Uds(#[from] UdsIncomingStreamError),
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server {
    config: Config,
    inner: InnerServer,
}

impl Server {
    pub async fn init(config: Config) -> Result<Self> {
        let inner = InnerServer::create_with(&config).await?;
        Ok(Self { config, inner })
    }

    pub async fn run(self) -> Result<()> {
        match self.inner {
            InnerServer::Http(server) => server.await?,
            InnerServer::Uds(server) => server.await?,
        }

        Ok(())
    }

    /// Gets a reference to the server's config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// If the server is an HTTP variant, returns the inner instance, otherwise returns `None`.
    pub fn as_http(&self) -> Option<&axum::Server<AddrIncoming, IntoMakeService<BoxRoute>>> {
        if let InnerServer::Http(ref server) = self.inner {
            Some(server)
        } else {
            None
        }
    }

    /// If the server is a UDS variant, returns the inner instance, otherwise returns `None`.
    pub fn as_uds(&self) -> Option<UdsIncomingStreamServer> {
        if let InnerServer::Uds(ref server) = self.inner {
            let path = match self.config.incoming_stream().as_unix_domain_socket() {
                Some(path) => path,
                None => return None,
            };
            Some(UdsIncomingStreamServer(server, path))
        } else {
            None
        }
    }
}

/// Wraps a UDS server to allow for a `local_path` method.
pub struct UdsIncomingStreamServer<'a>(
    &'a axum::Server<UdsIncomingStream, IntoMakeService<BoxRoute>>,
    &'a Path,
);

impl<'a> UdsIncomingStreamServer<'a> {
    pub fn local_path(&self) -> PathBuf {
        self.1.into()
    }
}

impl<'a> Deref for UdsIncomingStreamServer<'a> {
    type Target = &'a axum::Server<UdsIncomingStream, IntoMakeService<BoxRoute>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum InnerServer {
    Http(axum::Server<AddrIncoming, IntoMakeService<BoxRoute>>),
    Uds(axum::Server<UdsIncomingStream, IntoMakeService<BoxRoute>>),
}

impl InnerServer {
    async fn create_with(config: &Config) -> Result<Self> {
        let routes = routes(config)
            // TODO(fnichol): customize http tracing further, using:
            // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            )
            .boxed();

        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                let inner = axum::Server::bind(socket_addr).serve(routes.into_make_service());
                Ok(Self::Http(inner))
            }
            IncomingStream::UnixDomainSocket(path) => {
                info!("binding to Unix domain socket; path={}", path.display());
                let inner = axum::Server::builder(UdsIncomingStream::create(path).await?)
                    .serve(routes.into_make_service());
                Ok(Self::Uds(inner))
            }
        }
    }
}
