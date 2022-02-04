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
    CodeGenerationRequest, CodeGenerationResultSuccess, FunctionResult, Message, ProgressMessage,
};

pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

pub fn execute<T>(
    stream: WebSocketStream<T>,
    request: CodeGenerationRequest,
) -> CodeGenerationExecution<T> {
    CodeGenerationExecution { stream, request }
}

#[derive(Debug, Error)]
pub enum CodeGenerationExecutionError {
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
    MessageBeforeStart(Message<CodeGenerationResultSuccess>),
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

type Result<T> = std::result::Result<T, CodeGenerationExecutionError>;

#[derive(Debug)]
pub struct CodeGenerationExecution<T> {
    stream: WebSocketStream<T>,
    request: CodeGenerationRequest,
}

impl<T> CodeGenerationExecution<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn start(mut self) -> Result<CodeGenerationExecutionStarted<T>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(CodeGenerationExecutionError::JSONDeserialize)?;
                match msg {
                    Message::Start => {
                        // received correct message, so proceed
                    }
                    unexpected => {
                        return Err(CodeGenerationExecutionError::MessageBeforeStart(unexpected))
                    }
                }
            }
            Some(Ok(unexpected)) => {
                return Err(CodeGenerationExecutionError::UnexpectedMessageType(
                    unexpected,
                ))
            }
            Some(Err(err)) => return Err(CodeGenerationExecutionError::WSReadIO(err)),
            None => return Err(CodeGenerationExecutionError::WSClosedBeforeStart),
        }

        let msg = self
            .request
            .serialize_to_string()
            .map_err(CodeGenerationExecutionError::JSONSerialize)?;
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(CodeGenerationExecutionError::WSSendIO)?;

        Ok(self.into())
    }
}

impl<T> From<CodeGenerationExecution<T>> for CodeGenerationExecutionStarted<T> {
    fn from(value: CodeGenerationExecution<T>) -> Self {
        Self {
            stream: value.stream,
            result: None,
        }
    }
}

#[derive(Debug)]
pub struct CodeGenerationExecutionStarted<T> {
    stream: WebSocketStream<T>,
    result: Option<FunctionResult<CodeGenerationResultSuccess>>,
}

impl<T> CodeGenerationExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn finish(self) -> Result<FunctionResult<CodeGenerationResultSuccess>> {
        CodeGenerationExecutionClosing::try_from(self)?
            .finish()
            .await
    }
}

impl<T> Stream for CodeGenerationExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    type Item = Result<ProgressMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream.next()).poll(cx) {
            // We successfully got a websocket text message
            Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(CodeGenerationExecutionError::JSONDeserialize)?;
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
                            Poll::Ready(Some(Err(CodeGenerationExecutionError::FinishBeforeResult)))
                        }
                    }
                    // We got an unexpected message
                    unexpected => Poll::Ready(Some(Err(
                        CodeGenerationExecutionError::MessageBeforeStart(unexpected),
                    ))),
                }
            }
            // We successfully got an unexpected websocket message type that was not text
            Poll::Ready(Some(Ok(unexpected))) => Poll::Ready(Some(Err(
                CodeGenerationExecutionError::UnexpectedMessageType(unexpected),
            ))),
            // We failed to get the next websocket message
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(CodeGenerationExecutionError::WSReadIO(err))))
            }
            // We see the end of the websocket stream, but finish was never sent
            Poll::Ready(None) => Poll::Ready(Some(Err(
                CodeGenerationExecutionError::WSClosedBeforeFinish,
            ))),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct CodeGenerationExecutionClosing<T> {
    stream: WebSocketStream<T>,
    result: FunctionResult<CodeGenerationResultSuccess>,
}

impl<T> TryFrom<CodeGenerationExecutionStarted<T>> for CodeGenerationExecutionClosing<T> {
    type Error = CodeGenerationExecutionError;

    fn try_from(value: CodeGenerationExecutionStarted<T>) -> Result<Self> {
        match value.result {
            Some(result) => Ok(Self {
                stream: value.stream,
                result,
            }),
            None => Err(Self::Error::ClosingWithoutResult),
        }
    }
}

impl<T> CodeGenerationExecutionClosing<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn finish(mut self) -> Result<FunctionResult<CodeGenerationResultSuccess>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(self.result),
            Some(Ok(unexpected)) => {
                Err(CodeGenerationExecutionError::MessageAfterFinish(unexpected))
            }
            Some(Err(err)) => Err(CodeGenerationExecutionError::WSReadIO(err)),
        }
    }
}
