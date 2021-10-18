use std::{io, net::SocketAddr, path::PathBuf};

use axum::routing::{BoxRoute, IntoMakeService};
use hyper::server::{accept::Accept, conn::AddrIncoming};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    signal::unix,
    sync::{mpsc, oneshot},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{error, info};

use super::{routes, Config, IncomingStream, UdsIncomingStream, UdsIncomingStreamError};

#[derive(Debug, Error)]
pub enum ServerError {
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
    inner: axum::Server<I, IntoMakeService<BoxRoute>>,
    socket: S,
    shutdown_rx: oneshot::Receiver<()>,
}

impl Server<(), ()> {
    pub fn http(config: Config) -> Result<Server<AddrIncoming, SocketAddr>> {
        match config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                let (service, shutdown_rx) = build_service(&config)?;

                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                let inner = axum::Server::bind(&socket_addr).serve(service);
                let socket = inner.local_addr();

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong => return Err(ServerError::WrongIncomingStream("http", wrong.clone())),
        }
    }

    pub async fn uds(config: Config) -> Result<Server<UdsIncomingStream, PathBuf>> {
        match config.incoming_stream() {
            IncomingStream::UnixDomainSocket(path) => {
                let (service, shutdown_rx) = build_service(&config)?;

                info!("binding to Unix domain socket; path={}", path.display());
                let inner =
                    axum::Server::builder(UdsIncomingStream::create(path).await?).serve(service);
                let socket = path.clone();

                Ok(Server {
                    config,
                    inner,
                    socket,
                    shutdown_rx,
                })
            }
            wrong => return Err(ServerError::WrongIncomingStream("http", wrong.clone())),
        }
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

fn build_service(config: &Config) -> Result<(IntoMakeService<BoxRoute>, oneshot::Receiver<()>)> {
    let (limit_request_shutdown_tx, limit_request_shutdown_rx) = mpsc::channel::<()>(4);

    let routes = routes(config, limit_request_shutdown_tx)
        // TODO(fnichol): customize http tracing further, using:
        // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .boxed();

    let graceful_shutdown_rx = prepare_graceful_shutdown(limit_request_shutdown_rx)?;

    Ok((routes.into_make_service(), graceful_shutdown_rx))
}

fn prepare_graceful_shutdown(
    mut internal_graceful_shutdown_rx: mpsc::Receiver<()>,
) -> Result<oneshot::Receiver<()>> {
    let (graceful_shutdown_tx, graceful_shutdown_rx) = oneshot::channel::<()>();
    let mut sigterm_stream =
        unix::signal(unix::SignalKind::terminate()).map_err(ServerError::Signal)?;

    tokio::spawn(async move {
        fn send_graceful_shutdown(tx: oneshot::Sender<()>) {
            if let Err(_) = tx.send(()) {
                error!("the server graceful shutdown receiver has already dropped");
            }
        }

        tokio::select! {
            _ = sigterm_stream.recv() => {
                info!("received SIGTERM signal, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx);
            }
            _ = internal_graceful_shutdown_rx.recv() => {
                info!("received internal shutdown, performing graceful shutdown");
                send_graceful_shutdown(graceful_shutdown_tx);
            }
        };
    });

    Ok(graceful_shutdown_rx)
}
