use std::{
    path::{Path, PathBuf},
    task::{Context, Poll},
};

use futures::ready;
use hyper::server::accept::Accept;
use thiserror::Error;

use vsock::{VsockAddr,VsockListener,VsockStream};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum VsockIncomingStreamError {

    #[error("failed to bind to vsock: {1}")]
    Bind(#[source] std::io::Error, VsockAddr),

    //#[error("failed to create parent path for unix domain socket")]
    // CreateParentPath(#[source] std::io::Error),
    //#[error("IO error")]
    //IO(#[from] std::io::Error),
    //#[error("parent path not found for unix domain socket: {0}")]
    //ParentPathNotFound(PathBuf),
}

type Result<T> = std::result::Result<T, VsockIncomingStreamError>;

pub struct VsockIncomingStream {
    vsock: VsockListener,
}

// Change this to Port, so that Vsock can pick up the port and translate onto the host v.sock
impl VsockIncomingStream {
    pub async fn create(addr: VsockAddr) -> Result<Self> {
        let vsock = VsockListener::bind(&addr)
            .map_err(|err| VsockIncomingStreamError::Bind(err, addr))?;

        Ok(Self { vsock })
    }
}

impl Accept for VsockIncomingStream {
    type Conn = VsockStream;
    type Error = VsockIncomingStreamError;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>
    ) -> Poll<Option<Result<Self::Conn>>> {
        let (stream, _addr) = self.vsock.accept()?;
        Poll::Ready(Some(Ok(stream)))
    }
}
