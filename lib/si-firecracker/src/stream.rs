use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use nix::unistd::{chown, Gid, Uid};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs,
    io::{copy_bidirectional, AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream, UnixListener},
    select,
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, DropGuard};
use tokio_vsock::{VsockAddr, VsockStream, VMADDR_CID_HOST};

const UID_BASE: u32 = 5000;
const GID: u32 = 10000;
const DEFAULT_OTEL_PORT: u16 = 4317;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum StreamForwarderError {
    #[error("failed to bind to socket: {1}")]
    Bind(#[source] std::io::Error, PathBuf),
    #[error("chown error: {0}")]
    Chown(#[source] nix::errno::Errno),
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Unix Read error: {0}")]
    Read(#[source] std::io::Error),
    #[error("TCP Stream error: {0}")]
    Tcp(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, StreamForwarderError>;

#[derive(Debug)]
pub struct UnixStreamForwarder {
    source: UnixListener,
    port: u16,
    token: CancellationToken,
}

impl UnixStreamForwarder {
    pub async fn new(in_path: impl AsRef<Path>, id: u32) -> Result<Self> {
        let port = DEFAULT_OTEL_PORT;
        let mut path = in_path.as_ref().to_path_buf();
        path.push(format!("v.sock_{port}"));
        // cleanup just in case the socket already exists somehow
        let _ignored = fs::remove_file(path.clone()).await;

        let source = UnixListener::bind(&path)
            .map_err(|err| StreamForwarderError::Bind(err, path.clone()))?;
        // the jailer runs as a specific user and that user must own the socket
        chown_for_id(path, id)?;

        let token = CancellationToken::new();

        Ok(Self {
            source,
            port,
            token,
        })
    }

    #[allow(clippy::let_underscore_future)] // These needs to just run in the background forever.
    pub async fn start(self) -> Result<StreamForwarderHandle> {
        debug!(port = %self.port, "starting uds -> tcp forwarder");
        let token = self.token.clone();
        let handle = tokio::spawn(self.unix_stream_accept_connections_task());

        Ok(StreamForwarderHandle {
            handle,
            drop_guard: token.drop_guard(),
        })
    }

    async fn unix_stream_accept_connections_task(self) {
        let Self {
            source,
            port,
            token,
        } = self;

        loop {
            select! {
                // Possible new incoming connection from the listener
                result = source.accept() => {
                    match result {
                        // Sucessfully received new incoming connection
                        Ok((uds_stream, _)) => {
                            match TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port)).await
                            {
                                // Connection is successful
                                Ok(tcp_stream) => {
                                    tokio::spawn(
                                        read_and_forward(uds_stream, tcp_stream, token.clone())
                                    );
                                }
                                // Error while opening connection
                                Err(err) => {
                                    warn!(si.error = ?err, "error opening tcp connection");
                                }
                            }
                        }
                        // I/O error accepting incoming connection
                        Err(err) => {
                            warn!(si.error = ?err, "error accepting incoming connection");
                        }
                    }
                }
                // Cancellation token has fired, time to shut down task
                _ = token.cancelled() => {
                    trace!("stream accept connections task received cancellation");
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct StreamForwarderHandle {
    handle: JoinHandle<()>,
    drop_guard: DropGuard,
}

impl StreamForwarderHandle {
    pub async fn shutdown(self) -> Result<()> {
        self.drop_guard.disarm().cancel();
        self.handle.await.map_err(StreamForwarderError::Join)
    }
}

#[derive(Debug)]
pub struct TcpStreamForwarder {
    source: TcpListener,
    port: u16,
    token: CancellationToken,
}

impl TcpStreamForwarder {
    pub async fn new() -> Result<Self> {
        let port = DEFAULT_OTEL_PORT;
        let addr = format!("127.0.0.1:{}", port);
        let source = TcpListener::bind(addr)
            .await
            .map_err(StreamForwarderError::Tcp)?;

        let token = CancellationToken::new();

        Ok(Self {
            source,
            port,
            token,
        })
    }

    pub async fn start(self) -> Result<StreamForwarderHandle> {
        debug!(port = %self.port, "starting tcp -> vsock forwarder");
        let token = self.token.clone();
        let handle = tokio::spawn(self.tcp_stream_accept_connections_task());

        Ok(StreamForwarderHandle {
            handle,
            drop_guard: token.drop_guard(),
        })
    }

    async fn tcp_stream_accept_connections_task(self) {
        let Self {
            source,
            port,
            token,
        } = self;

        loop {
            select! {
                // Possible new incoming connection from the listener
                result = source.accept() => {
                    match result {
                        // Sucessfully received new incoming connection
                        Ok((tcp_stream, _)) => {
                            match VsockStream::connect(VsockAddr::new(VMADDR_CID_HOST, port.into()))
                                .await
                            {
                                // Connection is successful
                                Ok(vsock_stream) => {
                                    tokio::spawn(read_and_forward(
                                        tcp_stream,
                                        vsock_stream,
                                        token.clone(),
                                    ));
                                }
                                // Error while opening connection
                                Err(err) => {
                                    warn!(si.error = ?err, "error opening vsock connection");
                                }
                            }
                        }
                        // I/O error accepting incoming connection
                        Err(err) => {
                            warn!(si.error = ?err, "error accepting incoming connection");
                        }
                    }
                }
                // Cancellation token has fired, time to shut down task
                _ = token.cancelled() => {
                    trace!("stream accept connections task received cancellation");
                    break;
                }
            }
        }
    }
}

async fn read_and_forward<StreamA, StreamB>(
    mut a: StreamA,
    mut b: StreamB,
    token: CancellationToken,
) where
    StreamA: AsyncRead + AsyncWrite + Unpin + 'static,
    StreamB: AsyncRead + AsyncWrite + Unpin + 'static,
{
    select! {
        result = copy_bidirectional(&mut a, &mut b) => {
            if let Err(err) = result {
                warn!(si.error = ?err, "error while fowarding streams, aborting task");
            }
        }
        _ = token.cancelled() => {
            trace!("stream forwarding task received cancellation");
        }
    }
}

fn chown_for_id(path: PathBuf, id: u32) -> Result<()> {
    let uid = Uid::from_raw(UID_BASE + id);
    let gid = Gid::from_raw(GID);
    chown(&path, Some(uid), Some(gid)).map_err(StreamForwarderError::Chown)?;
    Ok(())
}
