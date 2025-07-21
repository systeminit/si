use std::path::{
    Path,
    PathBuf,
};

use nix::unistd::{
    Gid,
    Uid,
    chown,
};
use thiserror::Error;
use tokio::{
    fs,
    io::{
        AsyncRead,
        AsyncWrite,
        copy_bidirectional,
    },
    net::{
        TcpListener,
        TcpStream,
        UnixListener,
    },
    task::JoinHandle,
};
use tokio_vsock::{
    VMADDR_CID_HOST,
    VsockAddr,
    VsockStream,
};
use tracing::debug;

const UID_BASE: u32 = 5000;
const GID: u32 = 10000;
const DEFAULT_OTEL_PORT: u32 = 4317;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum StreamForwarderError {
    #[error("failed to accept stream: {0}")]
    Accept(#[source] std::io::Error),
    #[error("failed to bind to socket: {1}")]
    Bind(#[source] std::io::Error, PathBuf),
    #[error("chown error: {0}")]
    Chown(#[source] nix::errno::Errno),
    #[error("stream copy error: {0}")]
    Copy(#[source] std::io::Error),
    #[error("Unix Read error: {0}")]
    Read(#[source] std::io::Error),
    #[error("TCP Stream error: {0}")]
    Tcp(#[source] std::io::Error),
    #[error("Vsock Stream error: {0}")]
    Vsock(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, StreamForwarderError>;

#[derive(Debug)]
pub struct UnixStreamForwarder {
    source: UnixListener,
    port: u32,
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

        Ok(Self { source, port })
    }

    #[allow(clippy::let_underscore_future)] // These needs to just run in the background forever.
    pub async fn start(self) -> Result<()> {
        debug!(port = %self.port, "starting uds -> tcp forwarder");
        let _: JoinHandle<Result<()>> = tokio::spawn(async move {
            loop {
                let (uds_stream, _) = self
                    .source
                    .accept()
                    .await
                    .map_err(StreamForwarderError::Accept)?;

                let tcp_stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))
                    .await
                    .map_err(StreamForwarderError::Tcp)?;

                tokio::spawn(read_and_forward(uds_stream, tcp_stream));
            }
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct TcpStreamForwarder {
    source: TcpListener,
    port: u32,
}

impl TcpStreamForwarder {
    pub async fn new() -> Result<Self> {
        let port = DEFAULT_OTEL_PORT;
        let addr = format!("127.0.0.1:{port}");
        let source = TcpListener::bind(addr)
            .await
            .map_err(StreamForwarderError::Tcp)?;

        Ok(Self { source, port })
    }

    #[allow(clippy::let_underscore_future)] // These needs to just run in the background forever.
    pub async fn start(self) -> Result<()> {
        debug!(port = %self.port, "starting tcp -> vsock forwarder");
        let _: JoinHandle<Result<()>> = tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = self
                    .source
                    .accept()
                    .await
                    .map_err(StreamForwarderError::Accept)?;

                let vsock_stream = VsockStream::connect(VsockAddr::new(VMADDR_CID_HOST, self.port))
                    .await
                    .map_err(StreamForwarderError::Vsock)?;

                tokio::spawn(read_and_forward(tcp_stream, vsock_stream));
            }
        });
        Ok(())
    }
}
async fn read_and_forward<StreamA, StreamB>(mut a: StreamA, mut b: StreamB) -> Result<()>
where
    StreamA: AsyncRead + AsyncWrite + Unpin + 'static,
    StreamB: AsyncRead + AsyncWrite + Unpin + 'static,
{
    copy_bidirectional(&mut a, &mut b)
        .await
        .map_err(StreamForwarderError::Copy)?;

    Ok(())
}

fn chown_for_id(path: PathBuf, id: u32) -> Result<()> {
    let uid = Uid::from_raw(UID_BASE + id);
    let gid = Gid::from_raw(GID);
    chown(&path, Some(uid), Some(gid)).map_err(StreamForwarderError::Chown)?;
    Ok(())
}
