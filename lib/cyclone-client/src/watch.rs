use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Stream, StreamExt};
use futures_lite::FutureExt;
use hyper::client::connect::Connection;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time,
};
use tokio_tungstenite::{tungstenite::Message as WebSocketMessage, WebSocketStream};

pub fn watch<T>(stream: WebSocketStream<T>, ping_wait_timeout: Duration) -> Watch<T> {
    Watch {
        stream,
        ping_wait_timeout,
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WatchError {
    #[error("timeout while waiting to read next message")]
    Timeout(#[from] time::error::Elapsed),
    #[error("unexpected websocket message type: {0}")]
    UnexpectedMessageType(WebSocketMessage),
    #[error("failed to close websocket")]
    WSClose(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("failed to read websocket message")]
    WSReadIO(#[source] tokio_tungstenite::tungstenite::Error),
}

type Result<T> = std::result::Result<T, WatchError>;

pub struct Watch<T> {
    stream: WebSocketStream<T>,
    ping_wait_timeout: Duration,
}

impl<T> Watch<T> {
    pub async fn start(self) -> Result<WatchStarted<T>> {
        Ok(self.into())
    }
}

impl<T> From<Watch<T>> for WatchStarted<T> {
    fn from(value: Watch<T>) -> Self {
        Self {
            stream: value.stream,
            ping_wait_timeout: value.ping_wait_timeout,
        }
    }
}

pub struct WatchStarted<T> {
    stream: WebSocketStream<T>,
    ping_wait_timeout: Duration,
}

impl<T> WatchStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn stop(mut self) -> Result<()> {
        self.stream.close(None).await.map_err(WatchError::WSClose)
    }
}

impl<T> Stream for WatchStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    type Item = Result<()>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match time::timeout(self.ping_wait_timeout, self.stream.next())
            .boxed()
            .poll(cx)
        {
            // We successfully got a WebSocket ping message
            Poll::Ready(Ok(Some(Ok(WebSocketMessage::Ping(_))))) => {
                trace!("received ping from server");
                Poll::Ready(Some(Ok(())))
            }
            // We got an unexpected message that wasn't a ping
            Poll::Ready(Ok(Some(Ok(unexpected)))) => {
                Poll::Ready(Some(Err(WatchError::UnexpectedMessageType(unexpected))))
            }
            // We failed to get the next WebSocket message
            Poll::Ready(Ok(Some(Err(err)))) => Poll::Ready(Some(Err(WatchError::WSReadIO(err)))),
            // We see the end of the WebSocket stream, so this stream
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            // We've timed out while waiting to read the next message
            Poll::Ready(Err(elapsed)) => Poll::Ready(Some(Err(WatchError::Timeout(elapsed)))),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}
