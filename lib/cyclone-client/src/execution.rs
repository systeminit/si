use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use cyclone_core::{FunctionResult, Message, ProgressMessage};
use futures::{Future, SinkExt, Stream, StreamExt};
use hyper::client::connect::Connection;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

pub fn execute<T, Request, Success>(
    stream: WebSocketStream<T>,
    request: Request,
) -> Execution<T, Request, Success> {
    Execution {
        stream,
        request,
        success_marker: PhantomData,
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ExecutionError<Success> {
    #[error("closing execution stream without a result")]
    ClosingWithoutResult,
    #[error("finish message received before result message was received")]
    FinishBeforeResult,
    #[error("execution error: failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("unexpected websocket message after finish was sent: {0}")]
    MessageAfterFinish(WebSocketMessage),
    #[error("unexpected message before start was sent: {0:?}")]
    MessageBeforeStart(Message<Success>),
    #[error("unexpected message: {0:?}")]
    UnexpectedMessage(Message<Success>),
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

#[derive(Debug)]
pub struct Execution<T, Request, Success> {
    stream: WebSocketStream<T>,
    request: Request,
    // Are we sure this is the right variance?
    success_marker: PhantomData<Success>,
}

impl<T, Request, Success> Execution<T, Request, Success>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    Success: DeserializeOwned,
    Request: Serialize,
{
    pub async fn start(mut self) -> Result<ExecutionStarted<T, Success>, ExecutionError<Success>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                println!("in start in client");
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(ExecutionError::JSONDeserialize)?;
                match msg {
                    Message::Start => {
                        // received correct message, so proceed
                    }
                    unexpected => return Err(ExecutionError::MessageBeforeStart(unexpected)),
                }
            }
            Some(Ok(unexpected)) => return Err(ExecutionError::UnexpectedMessageType(unexpected)),
            Some(Err(err)) => return Err(ExecutionError::WSReadIO(err)),
            None => return Err(ExecutionError::WSClosedBeforeStart),
        }

        let msg = serde_json::to_string(&self.request).map_err(ExecutionError::JSONSerialize)?;
        dbg!(msg.clone());
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(ExecutionError::WSSendIO)?;

        println!("we sent to the server");
        Ok(self.into())
    }
}

impl<T, Request, Success> From<Execution<T, Request, Success>> for ExecutionStarted<T, Success> {
    fn from(value: Execution<T, Request, Success>) -> Self {
        Self {
            stream: value.stream,
            result: None,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionStarted<T, Success> {
    stream: WebSocketStream<T>,
    result: Option<FunctionResult<Success>>,
}

impl<T, Success> ExecutionStarted<T, Success>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn finish(self) -> Result<FunctionResult<Success>, ExecutionError<Success>> {
        ExecutionClosing::try_from(self)?.finish().await
    }
}

impl<T, Success> Stream for ExecutionStarted<T, Success>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    Success: DeserializeOwned + std::marker::Unpin + std::fmt::Debug,
{
    type Item = Result<ProgressMessage, ExecutionError<Success>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        println!("entering poll_next in execution started");
        match Pin::new(&mut self.stream.next()).poll(cx) {
            // We successfully got a websocket text message
            Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                println!("got msg: {}", &json_str);
                let msg = Message::deserialize_from_str(&json_str)
                    .map_err(ExecutionError::JSONDeserialize)?;
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
                            Poll::Ready(Some(Err(ExecutionError::FinishBeforeResult)))
                        }
                    }
                    // We got an unexpected message
                    unexpected => {
                        Poll::Ready(Some(Err(ExecutionError::UnexpectedMessage(unexpected))))
                    }
                }
            }
            // We successfully got an unexpected websocket message type that was not text
            Poll::Ready(Some(Ok(unexpected))) => {
                Poll::Ready(Some(Err(ExecutionError::UnexpectedMessageType(unexpected))))
            }
            // We failed to get the next websocket message
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(ExecutionError::WSReadIO(err)))),
            // We see the end of the websocket stream, but finish was never sent
            Poll::Ready(None) => Poll::Ready(Some(Err(ExecutionError::WSClosedBeforeFinish))),
            // Not ready, so...not ready!
            Poll::Pending => {
                println!("pending");
                Poll::Pending
            }
        }
    }
}

#[derive(Debug)]
pub struct ExecutionClosing<T, Success> {
    stream: WebSocketStream<T>,
    result: FunctionResult<Success>,
}

impl<T, Success> TryFrom<ExecutionStarted<T, Success>> for ExecutionClosing<T, Success> {
    type Error = ExecutionError<Success>;

    fn try_from(value: ExecutionStarted<T, Success>) -> Result<Self, Self::Error> {
        match value.result {
            Some(result) => Ok(Self {
                stream: value.stream,
                result,
            }),
            None => Err(Self::Error::ClosingWithoutResult),
        }
    }
}

impl<T, Success> ExecutionClosing<T, Success>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn finish(mut self) -> Result<FunctionResult<Success>, ExecutionError<Success>> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(self.result),
            Some(Ok(unexpected)) => Err(ExecutionError::MessageAfterFinish(unexpected)),
            Some(Err(err)) => Err(ExecutionError::WSReadIO(err)),
        }
    }
}
