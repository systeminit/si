use futures::ready;
use hyper::server::accept::Accept;
use std::{
    path::{Path, PathBuf},
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::{
    fs,
    net::{UnixListener, UnixStream},
};

#[derive(Debug, Error)]
pub enum UDSIncomingStreamError {
    #[error("failed to bind to unix domain socket: {1}")]
    Bind(#[source] std::io::Error, PathBuf),
    #[error("failed to create parent path for unix domain socket")]
    CreateParentPath(#[source] std::io::Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("parent path not found for unix domain socket: {0}")]
    ParentPathNotFound(PathBuf),
}

pub struct UDSIncomingStream {
    uds: UnixListener,
}

impl UDSIncomingStream {
    pub async fn create(path: impl AsRef<Path>) -> Result<Self, UDSIncomingStreamError> {
        // File might not exist so don't worry about possible error
        let _ = fs::remove_file(path.as_ref()).await;
        fs::create_dir_all(path.as_ref().parent().ok_or_else(|| {
            UDSIncomingStreamError::ParentPathNotFound(path.as_ref().to_path_buf())
        })?)
        .await
        .map_err(UDSIncomingStreamError::CreateParentPath)?;

        let uds = UnixListener::bind(path.as_ref())
            .map_err(|err| UDSIncomingStreamError::Bind(err, path.as_ref().to_path_buf()))?;

        Ok(Self { uds })
    }
}

impl Accept for UDSIncomingStream {
    type Conn = UnixStream;
    type Error = UDSIncomingStreamError;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _addr) = ready!(self.uds.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}
