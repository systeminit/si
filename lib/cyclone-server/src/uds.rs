use std::{
    path::{Path, PathBuf},
    task::{Context, Poll},
};

use futures::ready;
use hyper::server::accept::Accept;
use thiserror::Error;
use tokio::{
    fs,
    net::{UnixListener, UnixStream},
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum UdsIncomingStreamError {
    #[error("failed to bind to unix domain socket: {1}")]
    Bind(#[source] std::io::Error, PathBuf),
    #[error("failed to create parent path for unix domain socket")]
    CreateParentPath(#[source] std::io::Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("parent path not found for unix domain socket: {0}")]
    ParentPathNotFound(PathBuf),
}

type Result<T> = std::result::Result<T, UdsIncomingStreamError>;

pub struct UdsIncomingStream {
    uds: UnixListener,
}

impl UdsIncomingStream {
    pub async fn create(path: impl AsRef<Path>) -> Result<Self> {
        // File might not exist so don't worry about possible error
        let _ignored = fs::remove_file(path.as_ref()).await;
        fs::create_dir_all(path.as_ref().parent().ok_or_else(|| {
            UdsIncomingStreamError::ParentPathNotFound(path.as_ref().to_path_buf())
        })?)
        .await
        .map_err(UdsIncomingStreamError::CreateParentPath)?;

        let uds = UnixListener::bind(path.as_ref())
            .map_err(|err| UdsIncomingStreamError::Bind(err, path.as_ref().to_path_buf()))?;

        Ok(Self { uds })
    }
}

impl Accept for UdsIncomingStream {
    type Conn = UnixStream;
    type Error = UdsIncomingStreamError;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn>>> {
        let (stream, _addr) = ready!(self.uds.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}
