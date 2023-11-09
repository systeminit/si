use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::routing::{IntoMakeService, Router};
use cyclone_core::{CycloneDecryptionKey, CycloneDecryptionKeyError};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use si_std::{CanonicalFile, CanonicalFileError};
use telemetry::{prelude::*, TelemetryLevel};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal::unix,
    sync::{mpsc, oneshot},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{
    routes::routes, state::AppState, Config, IncomingStream, UdsIncomingStream,
    UdsIncomingStreamError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error(transparent)]
    DecryptionKey(#[from] CycloneDecryptionKeyError),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error("UDS incoming stream error")]
    Uds(#[from] UdsIncomingStreamError),
    #[error("wrong incoming stream for {0} server: {1:?}")]
    WrongIncomingStream(&'static str, IncomingStream),
}

type Result<T> = std::result::Result<T, ServerError>;

pub struct Server<I, S> {
    config: Config,
    inner: axum::Server<I, IntoMakeService<Router>>,
    socket: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server<(), ()> {
    pub fn http(
        config: Config,
        telemetry_level: Box<dyn TelemetryLevel>,
        decryption_key: CycloneDecryptionKey,
    ) -> Result<Server<AddrIncoming, SocketAddr>> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx) =
                    build_service(&config, telemetry_level, decryption_key)?;

                debug!(socket = %socket_addr, "binding an http server");
                let inner = axum::Server::bind(socket_addr).serve(service);
                let socket = inner.local_addr();
                info!(socket = %socket, "http server serving");

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong @ IncomingStream::UnixDomainSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    pub async fn uds(
        config: Config,
        telemetry_level: Box<dyn TelemetryLevel>,
        decryption_key: CycloneDecryptionKey,
    ) -> Result<Server<UdsIncomingStream, PathBuf>> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx) =
                    build_service(&config, telemetry_level, decryption_key)?;

                debug!(socket = %path.display(), "binding a unix domain server");
                let inner =
                    axum::Server::builder(UdsIncomingStream::create(path).await?).serve(service);
                let socket = path.clone();
                info!(socket = %socket.display(), "unix domain server serving");

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong @ IncomingStream::HTTPSocket(_) => {
                Err(ServerError::WrongIncomingStream("http", wrong.clone()))
            }
        }
    }

    pub async fn load_decryption_key(key_path: &Path) -> Result<CycloneDecryptionKey> {
        // Ensure the key path is canonicalized and exists
        let path = CanonicalFile::try_from(key_path)?;

        let key = CycloneDecryptionKey::load(path.as_path()).await?;
        Ok(key)
    }
}

impl<I, IO, IE, S> Server<I, S>
where
    I: Accept<Conn = IO, Error = IE>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn run(self) -> Result<()> {
        let shutdown_rx = self.shutdown_rx;

        self.inner
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .map_err(Into::into)
    }

    /// Gets a reference to the server's config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Gets a reference to the server's locally bound socket.
    pub fn local_socket(&self) -> &S {
        &self.socket
    }
}

fn build_service(
    config: &Config,
    telemetry_level: Box<dyn TelemetryLevel>,
    decryption_key: CycloneDecryptionKey,
) -> Result<(IntoMakeService<Router>, oneshot::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel(4);

    let state = AppState::new(config.lang_server_path(), decryption_key, telemetry_level);

    let routes = routes(config, state, shutdown_tx)
        // TODO(fnichol): customize http tracing further, using:
        // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

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
