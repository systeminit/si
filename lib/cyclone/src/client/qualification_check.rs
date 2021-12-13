use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, SinkExt, Stream, StreamExt};
use hyper::client::connect::Connection;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

use crate::qualification_check::{
    QualificationCheckExecutingMessage, QualificationCheckMessage, QualificationCheckRequest,
    QualificationCheckResult,
};

pub fn execute<T>(
    stream: WebSocketStream<T>,
    request: QualificationCheckRequest,
) -> QualificationCheckExecution<T> {
    QualificationCheckExecution { stream, request }
}

#[derive(Debug, Error)]
pub enum QualificationCheckExecutionError {
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
    MessageBeforeStart(QualificationCheckMessage),
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

type Result<T> = std::result::Result<T, QualificationCheckExecutionError>;

#[derive(Debug)]
pub struct QualificationCheckExecution<T> {
    stream: WebSocketStream<T>,
    request: QualificationCheckRequest,
}

impl<T> QualificationCheckExecution<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn start(mut self) -> Result<QualificationCheckExecutionStarted<T>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                let msg = QualificationCheckMessage::deserialize_from_str(&json_str)
                    .map_err(QualificationCheckExecutionError::JSONDeserialize)?;
                match msg {
                    QualificationCheckMessage::Start => {
                        // received correct message, so proceed
                    }
                    unexpected => {
                        return Err(QualificationCheckExecutionError::MessageBeforeStart(
                            unexpected,
                        ))
                    }
                }
            }
            Some(Ok(unexpected)) => {
                return Err(QualificationCheckExecutionError::UnexpectedMessageType(
                    unexpected,
                ))
            }
            Some(Err(err)) => return Err(QualificationCheckExecutionError::WSReadIO(err)),
            None => return Err(QualificationCheckExecutionError::WSClosedBeforeStart),
        }

        let msg = self
            .request
            .serialize_to_string()
            .map_err(QualificationCheckExecutionError::JSONSerialize)?;
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(QualificationCheckExecutionError::WSSendIO)?;

        Ok(self.into())
    }
}

impl<T> From<QualificationCheckExecution<T>> for QualificationCheckExecutionStarted<T> {
    fn from(value: QualificationCheckExecution<T>) -> Self {
        Self {
            stream: value.stream,
            result: None,
        }
    }
}

#[derive(Debug)]
pub struct QualificationCheckExecutionStarted<T> {
    stream: WebSocketStream<T>,
    result: Option<QualificationCheckResult>,
}

impl<T> QualificationCheckExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn finish(self) -> Result<QualificationCheckResult> {
        QualificationCheckExecutionClosing::try_from(self)?
            .finish()
            .await
    }
}

impl<T> Stream for QualificationCheckExecutionStarted<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    type Item = Result<QualificationCheckExecutingMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream.next()).poll(cx) {
            // We successfully got a websocket text message
            Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                let msg = QualificationCheckMessage::deserialize_from_str(&json_str)
                    .map_err(QualificationCheckExecutionError::JSONDeserialize)?;
                match msg {
                    // We got a heartbeat message, pass it on
                    QualificationCheckMessage::Heartbeat => {
                        Poll::Ready(Some(Ok(QualificationCheckExecutingMessage::Heartbeat)))
                    }
                    // We got an output message, pass it on
                    QualificationCheckMessage::OutputStream(output_stream) => {
                        Poll::Ready(Some(Ok(QualificationCheckExecutingMessage::OutputStream(
                            output_stream,
                        ))))
                    }
                    // We got a funtion result message, save it and continue
                    QualificationCheckMessage::Result(function_result) => {
                        self.result = Some(function_result);
                        // TODO(fnichol): what is the right return here??
                        // (future fnichol): hey buddy! pretty sure you can:
                        // `cx.waker().wake_by_ref()` before returning Poll::Ready which immediatly
                        // re-wakes this stream to maybe pop another item off. cool huh? I think
                        // you're learning and that's great.
                        Poll::Ready(Some(Ok(QualificationCheckExecutingMessage::Heartbeat)))
                        //Poll::Pending
                    }
                    // We got a finish message
                    QualificationCheckMessage::Finish => {
                        if self.result.is_some() {
                            // If we have saved the result, then close this stream out
                            Poll::Ready(None)
                        } else {
                            // Otherwise we got a finish before seeing the result
                            Poll::Ready(Some(Err(
                                QualificationCheckExecutionError::FinishBeforeResult,
                            )))
                        }
                    }
                    // We got an unexpected message
                    unexpected => Poll::Ready(Some(Err(
                        QualificationCheckExecutionError::MessageBeforeStart(unexpected),
                    ))),
                }
            }
            // We successfully got an unexpected websocket message type that was not text
            Poll::Ready(Some(Ok(unexpected))) => Poll::Ready(Some(Err(
                QualificationCheckExecutionError::UnexpectedMessageType(unexpected),
            ))),
            // We failed to get the next websocket message
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(QualificationCheckExecutionError::WSReadIO(err))))
            }
            // We see the end of the websocket stream, but finish was never sent
            Poll::Ready(None) => Poll::Ready(Some(Err(
                QualificationCheckExecutionError::WSClosedBeforeFinish,
            ))),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct QualificationCheckExecutionClosing<T> {
    stream: WebSocketStream<T>,
    result: QualificationCheckResult,
}

impl<T> TryFrom<QualificationCheckExecutionStarted<T>> for QualificationCheckExecutionClosing<T> {
    type Error = QualificationCheckExecutionError;

    fn try_from(value: QualificationCheckExecutionStarted<T>) -> Result<Self> {
        match value.result {
            Some(result) => Ok(Self {
                stream: value.stream,
                result,
            }),
            None => Err(Self::Error::ClosingWithoutResult),
        }
    }
}

impl<T> QualificationCheckExecutionClosing<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn finish(mut self) -> Result<QualificationCheckResult> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(self.result),
            Some(Ok(unexpected)) => Err(QualificationCheckExecutionError::MessageAfterFinish(
                unexpected,
            )),
            Some(Err(err)) => Err(QualificationCheckExecutionError::WSReadIO(err)),
        }
    }
}
