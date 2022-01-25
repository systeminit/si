use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, SinkExt, Stream, StreamExt};
use hyper::client::connect::Connection;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

use crate::{
    FunctionResult, Message, ProgressMessage, ResourceSyncRequest, ResourceSyncResultSuccess,
};

pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

pub fn execute<T>(
    stream: WebSocketStream<T>,
    request: ResourceSyncRequest,
) -> ResourceSyncExecution<T> {
    ResourceSyncExecution { stream, request }
}

#[derive(Debug, Error)]
pub enum ResourceSyncExecutionError {
    #[error("closing execution stream without a result")]
    ClosingWithoutResult,
    #[error("finish message received before result message was received")]
    FinishBeforeResult,
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("unexpected websocket message after finish was sent: {0}")]
    MessageAfterFinish(WebSocketMessage),
    #[error("unexpected qualification check message before start was sent: {0:?}")]
    MessageBeforeStart(Message<ResourceSyncResultSuccess>),
    #[error("unexpected websocket message type: {0}")]
    UnexpectedMessageType(WebSocketMessage),
    #[error("websocket stream is closed, but finish was not sent")]
    WSClosedBeforeFinish,
    #[error("websocket stream is closed, but start was not sent")]
    WSClosedBeforeStart,
    #[error("failed to read websocket message")]
    WSReadIO(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("failed to send websocket message")]
    WSSendIO(#[source] tokio_tungstenite::tungstenite::Error),
}

type Result<T> = std::result::Result<T, ResourceSyncExecutionError>;

#[derive(Debug)]
pub struct ResourceSyncExecution<T> {
    stream: WebSocketStream<T>,
    request: ResourceSyncRequest,
}

impl<T> ResourceSyncExecution<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn start(mut self) -> Result<ResourceSyncExecutionStarted<T>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(ResourceSyncExecutionError::JSONDeserialize)?;
                match msg {
                    Message::Start => {
                        // received correct message, so proceed
                    }
                    unexpected => {
                        return Err(ResourceSyncExecutionError::MessageBeforeStart(unexpected))
                    }
                }
            }
            Some(Ok(unexpected)) => {
                return Err(ResourceSyncExecutionError::UnexpectedMessageType(
                    unexpected,
                ))
            }
            Some(Err(err)) => return Err(ResourceSyncExecutionError::WSReadIO(err)),
            None => return Err(ResourceSyncExecutionError::WSClosedBeforeStart),
        }

        let msg = self
            .request
            .serialize_to_string()
            .map_err(ResourceSyncExecutionError::JSONSerialize)?;
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(ResourceSyncExecutionError::WSSendIO)?;

        Ok(self.into())
    }
}

impl<T> From<ResourceSyncExecution<T>> for ResourceSyncExecutionStarted<T> {
    fn from(value: ResourceSyncExecution<T>) -> Self {
        Self {
            stream: value.stream,
            result: None,
        }
    }
}

#[derive(Debug)]
pub struct ResourceSyncExecutionStarted<T> {
    stream: WebSocketStream<T>,
    result: Option<FunctionResult<ResourceSyncResultSuccess>>,
}

impl<T> ResourceSyncExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn finish(self) -> Result<FunctionResult<ResourceSyncResultSuccess>> {
        ResourceSyncExecutionClosing::try_from(self)?.finish().await
    }
}

impl<T> Stream for ResourceSyncExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    type Item = Result<ProgressMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream.next()).poll(cx) {
            // We successfully got a websocket text message
            Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(ResourceSyncExecutionError::JSONDeserialize)?;
                match msg {
                    // We got a heartbeat message, pass it on
                    Message::Heartbeat => Poll::Ready(Some(Ok(ProgressMessage::Heartbeat))),
                    // We got an output message, pass it on
                    Message::OutputStream(output_stream) => {
                        Poll::Ready(Some(Ok(ProgressMessage::OutputStream(output_stream))))
                    }
                    // We got a funtion result message, save it and continue
                    Message::Result(function_result) => {
                        self.result = Some(function_result);
                        // TODO(fnichol): what is the right return here??
                        // (future fnichol): hey buddy! pretty sure you can:
                        // `cx.waker().wake_by_ref()` before returning Poll::Ready which immediatly
                        // re-wakes this stream to maybe pop another item off. cool huh? I think
                        // you're learning and that's great.
                        Poll::Ready(Some(Ok(ProgressMessage::Heartbeat)))
                        //Poll::Pending
                    }
                    // We got a finish message
                    Message::Finish => {
                        if self.result.is_some() {
                            // If we have saved the result, then close this stream out
                            Poll::Ready(None)
                        } else {
                            // Otherwise we got a finish before seeing the result
                            Poll::Ready(Some(Err(ResourceSyncExecutionError::FinishBeforeResult)))
                        }
                    }
                    // We got an unexpected message
                    unexpected => Poll::Ready(Some(Err(
                        ResourceSyncExecutionError::MessageBeforeStart(unexpected),
                    ))),
                }
            }
            // We successfully got an unexpected websocket message type that was not text
            Poll::Ready(Some(Ok(unexpected))) => Poll::Ready(Some(Err(
                ResourceSyncExecutionError::UnexpectedMessageType(unexpected),
            ))),
            // We failed to get the next websocket message
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(ResourceSyncExecutionError::WSReadIO(err))))
            }
            // We see the end of the websocket stream, but finish was never sent
            Poll::Ready(None) => {
                Poll::Ready(Some(Err(ResourceSyncExecutionError::WSClosedBeforeFinish)))
            }
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct ResourceSyncExecutionClosing<T> {
    stream: WebSocketStream<T>,
    result: FunctionResult<ResourceSyncResultSuccess>,
}

impl<T> TryFrom<ResourceSyncExecutionStarted<T>> for ResourceSyncExecutionClosing<T> {
    type Error = ResourceSyncExecutionError;

    fn try_from(value: ResourceSyncExecutionStarted<T>) -> Result<Self> {
        match value.result {
            Some(result) => Ok(Self {
                stream: value.stream,
                result,
            }),
            None => Err(Self::Error::ClosingWithoutResult),
        }
    }
}

impl<T> ResourceSyncExecutionClosing<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn finish(mut self) -> Result<FunctionResult<ResourceSyncResultSuccess>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(self.result),
            Some(Ok(unexpected)) => Err(ResourceSyncExecutionError::MessageAfterFinish(unexpected)),
            Some(Err(err)) => Err(ResourceSyncExecutionError::WSReadIO(err)),
        }
    }
}
