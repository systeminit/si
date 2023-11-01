use std::task::{Context, Poll};

use futures::ready;
use hyper::server::accept::Accept;
use thiserror::Error;

use tokio_vsock::{VsockAddr, VsockListener, VsockStream};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum VsockIncomingStreamError {
    #[error("failed to bind to vsock: {1}")]
    Bind(#[source] std::io::Error, VsockAddr),
    #[error("IO error")]
    IO(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, VsockIncomingStreamError>;

pub struct VsockIncomingStream {
    vsock: VsockListener,
}

// Change this to Port, so that Vsock can pick up the port and translate onto the host v.sock
impl VsockIncomingStream {
    pub async fn create(addr: VsockAddr) -> Result<Self> {
        let vsock = VsockListener::bind(addr.cid(), addr.port())
            .map_err(|err| VsockIncomingStreamError::Bind(err, addr))?;

        Ok(Self { vsock })
    }
}

impl Accept for VsockIncomingStream {
    type Conn = VsockStream;
    type Error = VsockIncomingStreamError;

    fn poll_accept(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn>>> {
        let (stream, _addr) = ready!(self.vsock.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}
