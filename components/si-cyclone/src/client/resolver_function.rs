use std::{
    convert::TryFrom,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, SinkExt, Stream, StreamExt};
use hyper::client::connect::Connection;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

use crate::resolver_function::{
    FunctionResult, ResolverFunctionExecutingMessage, ResolverFunctionMessage,
    ResolverFunctionRequest,
};

pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

pub fn execute<T>(
    stream: WebSocketStream<T>,
    request: ResolverFunctionRequest,
) -> ResolverFunctionExecution<T> {
    ResolverFunctionExecution { stream, request }
}

#[derive(Debug, Error)]
pub enum ResolverFunctionExecutionError {
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
    #[error("unexpected resolver function message before start was sent: {0:?}")]
    MessageBeforeStart(ResolverFunctionMessage),
    #[error("websocket stream is closed, but finish was not sent")]
    WSClosedBeforeFinish,
    #[error("websocket stream is closed, but start was not sent")]
    WSClosedBeforeStart,
    #[error("failed to read websocket message")]
    WSReadIO(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("failed to send websocket message")]
    WSSendIO(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("unexpected websocket message type: {0}")]
    UnexpectedMessageType(WebSocketMessage),
}

type Result<T> = std::result::Result<T, ResolverFunctionExecutionError>;

#[derive(Debug)]
pub struct ResolverFunctionExecution<T> {
    stream: WebSocketStream<T>,
    request: ResolverFunctionRequest,
}

impl<T> ResolverFunctionExecution<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn start(mut self) -> Result<ResolverFunctionExecutionStarted<T>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                let msg = ResolverFunctionMessage::deserialize_from_str(&json_str)
                    .map_err(ResolverFunctionExecutionError::JSONDeserialize)?;
                match msg {
                    ResolverFunctionMessage::Start => {
                        // received correct message, so proceed
                    }
                    unexpected => {
                        return Err(ResolverFunctionExecutionError::MessageBeforeStart(
                            unexpected,
                        ))
                    }
                }
            }
            Some(Ok(unexpected)) => {
                return Err(ResolverFunctionExecutionError::UnexpectedMessageType(
                    unexpected,
                ))
            }
            Some(Err(err)) => return Err(ResolverFunctionExecutionError::WSReadIO(err)),
            None => return Err(ResolverFunctionExecutionError::WSClosedBeforeStart),
        }

        let msg = self
            .request
            .serialize_to_string()
            .map_err(ResolverFunctionExecutionError::JSONSerialize)?;
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(ResolverFunctionExecutionError::WSSendIO)?;

        Ok(self.into())
    }
}

impl<T> From<ResolverFunctionExecution<T>> for ResolverFunctionExecutionStarted<T> {
    fn from(value: ResolverFunctionExecution<T>) -> Self {
        Self {
            stream: value.stream,
            result: None,
        }
    }
}

#[derive(Debug)]
pub struct ResolverFunctionExecutionStarted<T> {
    stream: WebSocketStream<T>,
    result: Option<FunctionResult>,
}

impl<T> ResolverFunctionExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn finish(self) -> Result<FunctionResult> {
        ResolverFunctionExecutionClosing::try_from(self)?
            .finish()
            .await
    }
}

impl<T> Stream for ResolverFunctionExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    type Item = Result<ResolverFunctionExecutingMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream.next()).poll(cx) {
            // We successfully got a websocket text message
            Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                let msg = ResolverFunctionMessage::deserialize_from_str(&json_str)
                    .map_err(ResolverFunctionExecutionError::JSONDeserialize)?;
                match msg {
                    // We got a heartbeat message, pass it on
                    ResolverFunctionMessage::Heartbeat => {
                        Poll::Ready(Some(Ok(ResolverFunctionExecutingMessage::Heartbeat)))
                    }
                    // We got an output message, pass it on
                    ResolverFunctionMessage::OutputStream(output_stream) => Poll::Ready(Some(Ok(
                        ResolverFunctionExecutingMessage::OutputStream(output_stream),
                    ))),
                    // We got a funtion result message, save it and continue
                    ResolverFunctionMessage::FunctionResult(function_result) => {
                        self.result = Some(function_result);
                        // TODO(fnichol): what is the right return here??
                        Poll::Ready(Some(Ok(ResolverFunctionExecutingMessage::Heartbeat)))
                        //Poll::Pending
                    }
                    // We got a finish message
                    ResolverFunctionMessage::Finish => {
                        if self.result.is_some() {
                            // If we have saved the result, then close this stream out
                            Poll::Ready(None)
                        } else {
                            // Otherwise we got a finish before seeing the result
                            Poll::Ready(Some(Err(
                                ResolverFunctionExecutionError::FinishBeforeResult,
                            )))
                        }
                    }
                    // We got an unexpected message
                    unexpected => Poll::Ready(Some(Err(
                        ResolverFunctionExecutionError::MessageBeforeStart(unexpected),
                    ))),
                }
            }
            // We successfully got an unexpected websocket message type that was not text
            Poll::Ready(Some(Ok(unexpected))) => Poll::Ready(Some(Err(
                ResolverFunctionExecutionError::UnexpectedMessageType(unexpected),
            ))),
            // We failed to get the next websocket message
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(ResolverFunctionExecutionError::WSReadIO(err))))
            }
            // We see the end of the websocket stream, but finish was never sent
            Poll::Ready(None) => Poll::Ready(Some(Err(
                ResolverFunctionExecutionError::WSClosedBeforeFinish,
            ))),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct ResolverFunctionExecutionClosing<T> {
    stream: WebSocketStream<T>,
    result: FunctionResult,
}

impl<T> TryFrom<ResolverFunctionExecutionStarted<T>> for ResolverFunctionExecutionClosing<T> {
    type Error = ResolverFunctionExecutionError;

    fn try_from(value: ResolverFunctionExecutionStarted<T>) -> Result<Self> {
        match value.result {
            Some(result) => Ok(Self {
                stream: value.stream,
                result,
            }),
            None => Err(Self::Error::ClosingWithoutResult),
        }
    }
}

impl<T> ResolverFunctionExecutionClosing<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn finish(mut self) -> Result<FunctionResult> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(self.result),
            Some(Ok(unexpected)) => Err(ResolverFunctionExecutionError::MessageAfterFinish(
                unexpected,
            )),
            Some(Err(err)) => Err(ResolverFunctionExecutionError::WSReadIO(err)),
        }
    }
}
