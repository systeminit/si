use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use axum::routing::{IntoMakeService, Router};
use hyper::server::accept::Accept;
use telemetry::{prelude::*, TelemetryLevel};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal::unix,
    sync::{mpsc, oneshot},
};

use crate::{
    routes::routes, state::AppState, Config, IncomingStream, UdsIncomingStream,
    UdsIncomingStreamError,
};

#[cfg(target_os = "linux")]
use crate::{VsockIncomingStream, VsockIncomingStreamError};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error("UDS incoming stream error")]
    Uds(#[from] UdsIncomingStreamError),
    #[cfg(target_os = "linux")]
    #[error("Vsock incoming stream error")]
    Vsock(#[from] VsockIncomingStreamError),
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
}

type Result<T> = std::result::Result<T, ServerError>;

// Runnable trait which can be used as a trait object (i.e. `Box<dyn Runnable>`), containing a
// method which moves `self` (i.e. `fn run(self)`).
//
// See: https://users.rust-lang.org/t/need-explanation-on-how-to-avoid-this-move-out-of-a-box-dyn/98734/3
// See: https://quinedot.github.io/rust-learning/dyn-trait-box-impl.html
mod runnable {
    use super::Result;

    use async_trait::async_trait;

    #[async_trait]
    pub trait BoxedRunnable {
        async fn boxed_run(self: Box<Self>) -> Result<()>;
    }

    #[async_trait]
    pub trait Runnable: BoxedRunnable {
        async fn run(self) -> Result<()>;
    }

    #[async_trait]
    impl<T: Runnable + Send> BoxedRunnable for T {
        async fn boxed_run(self: Box<Self>) -> Result<()> {
            <Self as Runnable>::run(*self).await
        }
    }

    #[async_trait]
    impl Runnable for Box<dyn Runnable + Send + '_> {
        async fn run(self) -> Result<()> {
            <dyn Runnable as BoxedRunnable>::boxed_run(self).await
        }
    }
}

pub use runnable::Runnable;

pub struct Server {
    inner: Box<dyn Runnable + Send>,
    config: Config,
    socket: ServerSocket,
}

impl Server {
    pub async fn from_config(
        config: Config,
        telemetry_level: Box<dyn TelemetryLevel>,
    ) -> Result<Self> {
        let (service, shutdown_rx) = build_service(&config, telemetry_level)?;

        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                debug!(socket = %socket_addr, "binding an http server");
                let inner = axum::Server::bind(socket_addr).serve(service);
                let socket = inner.local_addr();
                info!(socket = %socket, "http server serving");

                Ok(Self {
                    inner: Box::new(InnerServer { inner, shutdown_rx }),
                    config,
                    socket: ServerSocket::SocketAddr(socket),
                })
            }
            IncomingStream::UnixDomainSocket(path) => {
                debug!(socket = %path.display(), "binding a unix domain server");
                let inner =
                    axum::Server::builder(UdsIncomingStream::create(path).await?).serve(service);
                let socket = path.clone();
                debug!(socket = %socket.display(), "unix domain server serving");

                Ok(Self {
                    inner: Box::new(InnerServer { inner, shutdown_rx }),
                    config,
                    socket: ServerSocket::DomainSocket(socket),
                })
            }
            #[cfg(target_os = "linux")]
            IncomingStream::VsockSocket(addr) => {
                debug!(socket = %addr, "binding a vsock server");
                let inner =
                    axum::Server::builder(VsockIncomingStream::create(*addr).await?).serve(service);
                let socket = *addr;
                info!(socket = %socket, "vsock server serving");

                Ok(Self {
                    inner: Box::new(InnerServer { inner, shutdown_rx }),
                    config,
                    socket: ServerSocket::VsockAddr(socket),
                })
            }
        }
    }

    /// Gets a reference to the server's config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Gets a reference to the server's locally bound socket.
    pub fn local_socket(&self) -> &ServerSocket {
        &self.socket
    }
}

#[async_trait]
impl Runnable for Server {
    async fn run(self) -> Result<()> {
        self.inner.run().await
    }
}

struct InnerServer<I> {
    inner: axum::Server<I, IntoMakeService<Router>>,
    shutdown_rx: oneshot::Receiver<()>,
}

#[async_trait]
impl<I, IO, IE> Runnable for InnerServer<I>
where
    I: Accept<Conn = IO, Error = IE> + Send + Sync,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    async fn run(self) -> Result<()> {
        let shutdown_rx = self.shutdown_rx;

        self.inner
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .map_err(Into::into)
    }
}

#[remain::sorted]
pub enum ServerSocket {
    DomainSocket(PathBuf),
    SocketAddr(SocketAddr),
    #[cfg(target_os = "linux")]
    VsockAddr(tokio_vsock::VsockAddr),
}

impl ServerSocket {
    pub fn as_domain_socket(&self) -> Option<&Path> {
        match self {
            Self::DomainSocket(pathbuf) => Some(pathbuf.as_path()),
            _ => None,
        }
    }

    pub fn as_socket_addr(&self) -> Option<&SocketAddr> {
        match self {
            Self::SocketAddr(addr) => Some(addr),
            _ => None,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn as_vsock_addr(&self) -> Option<&tokio_vsock::VsockAddr> {
        match self {
            Self::VsockAddr(addr) => Some(addr),
            _ => None,
        }
    }
}

fn build_service(
    config: &Config,
    telemetry_level: Box<dyn TelemetryLevel>,
) -> Result<(IntoMakeService<Router>, oneshot::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(4);

    let state = AppState::new(config.lang_server_path(), telemetry_level);

    let routes = routes(config, state, shutdown_tx);

    let graceful_shutdown_rx = prepare_graceful_shutdown(shutdown_rx)?;

    Ok((routes.into_make_service(), graceful_shutdown_rx))
}

fn prepare_graceful_shutdown(
    mut shutdown_rx: mpsc::Receiver<ShutdownSource>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(tx: oneshot::Sender<()>) {
            if tx.send(()).is_err() {
                error!("the server graceful shutdown receiver has already dropped");
            }
        }

        tokio::select! {
            _ = sigterm_stream.recv() => {
                trace!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx);
            }
            source = shutdown_rx.recv() => {
                trace!(
                    "received internal shutdown, performing graceful shutdown; source={:?}",
                    source,
                );
                send_graceful_shutdown(graceful_shutdown_tx);
            }
            else => {
                // All other arms are closed, nothing left to do but return
                trace!("returning from graceful shutdown with all select arms closed");
            }
        };
    });

    Ok(graceful_shutdown_rx)
}

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {
    LimitRequest,
    WatchTimeout,
}
