use core::fmt;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use cyclone_core::{CycloneRequest, CycloneRequestable, FunctionResult, Message, ProgressMessage};
use futures::{future::BoxFuture, Future, SinkExt, Stream, StreamExt};
use futures_lite::FutureExt as _;
use hyper::client::connect::Connection;
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use si_runtime::{DedicatedExecutor, DedicatedExecutorError};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub use tokio_tungstenite::tungstenite::Message as WebSocketMessage;

pub fn new_unstarted_execution<T, Request>(
    stream: WebSocketStream<T>,
    request: CycloneRequest<Request>,
) -> Execution<T, Request, Request::Response>
where
    Request: CycloneRequestable,
{
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
    #[error("dedicated executor error: {0}")]
    DedicatedExecutor(#[from] DedicatedExecutorError),
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
pub struct Execution<T, Request, Success>
where
    Request: CycloneRequestable,
{
    stream: WebSocketStream<T>,
    request: CycloneRequest<Request>,
    // Are we sure this is the right variance?
    success_marker: PhantomData<Success>,
}

impl<T, Request, Success> Execution<T, Request, Success>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
    Success: DeserializeOwned,
    Request: Serialize + CycloneRequestable + Send + 'static,
{
    pub async fn start(
        mut self,
        compute_executor: DedicatedExecutor,
    ) -> Result<ExecutionStarted<T, Success>, ExecutionError<Success>> {
        // As soon as we see the "start" message, we are good to go.
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
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

        // Once the start message has been seen on the stream, we can send the request.
        let msg = serde_json::to_string(&self.request).map_err(ExecutionError::JSONSerialize)?;
        //
        // // Alternative
        // let msg = compute_executor
        //     .spawn(async move { serde_json::to_string(&self.request) })
        //     .await?
        //     .map_err(ExecutionError::JSONSerialize)?;
        self.stream
            .send(WebSocketMessage::Text(msg))
            .await
            .map_err(ExecutionError::WSSendIO)?;

        Ok(ExecutionStarted {
            compute_executor,
            deserialize_fut: None,
            stream: self.stream,
            result: None,
        })
    }
}

pin_project! {
    pub struct ExecutionStarted<T, Success> {
        compute_executor: DedicatedExecutor,
        deserialize_fut: Option<Result<Message<Success>, ExecutionError<Success>>>,
        //
        // // Alternative
        // #[pin]
        // deserialize_fut: Option<
        //     BoxFuture<
        //         'static,
        //         Result<Result<Message<Success>, ExecutionError<Success>>, DedicatedExecutorError>,
        //     >,
        // >,
        #[pin]
        stream: WebSocketStream<T>,
        result: Option<FunctionResult<Success>>,
    }
}

impl<T, Success> fmt::Debug for ExecutionStarted<T, Success>
where
    T: fmt::Debug,
    Success: fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionStarted")
            .field("compute_executor", &self.compute_executor)
            .field("stream", &self.stream)
            .field("result", &self.result)
            .finish_non_exhaustive()
    }
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
    Success: DeserializeOwned + std::marker::Unpin + std::fmt::Debug + Send + 'static,
{
    type Item = Result<ProgressMessage, ExecutionError<Success>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // If there a deserialize future, then poll it first to see if a deserialized message
            // is ready
            if let Some(deserialize_fut) = this.deserialize_fut.take() {
                // match deserialize_fut.poll(cx) {
                //     // Message has finished deserializing
                //     Poll::Ready(msg_result) => {
                //         let msg = msg_result??;
                let msg = deserialize_fut?;

                match msg {
                    // We got a heartbeat message, pass it on
                    Message::Heartbeat => return Poll::Ready(Some(Ok(ProgressMessage::Heartbeat))),
                    // We got an output message, pass it on
                    Message::OutputStream(output_stream) => {
                        return Poll::Ready(Some(Ok(ProgressMessage::OutputStream(output_stream))))
                    }
                    // We got a funtion result message, save it and continue
                    Message::Result(function_result) => {
                        *this.result = Some(function_result);
                    }
                    // We got a finish message
                    Message::Finish => {
                        if this.result.is_some() {
                            // If we have saved the result, then close this stream out
                            return Poll::Ready(None);
                        } else {
                            // Otherwise we got a finish before seeing the result
                            return Poll::Ready(Some(Err(ExecutionError::FinishBeforeResult)));
                        }
                    }
                    // We got an unexpected message
                    unexpected => {
                        return Poll::Ready(Some(Err(ExecutionError::UnexpectedMessage(
                            unexpected,
                        ))));
                    }
                }

                //     }
                //     Poll::Pending => {
                //         // If we're still waiting on the deserialize, keep tracking this future
                //         *this.deserialize_fut = Some(deserialize_fut);
                //         return Poll::Pending;
                //     }
                // }
            }

            // Next, check the websocket stream for a message
            match this.stream.next().poll(cx) {
                // We successfully got a websocket text message
                Poll::Ready(Some(Ok(WebSocketMessage::Text(json_str)))) => {
                    *this.deserialize_fut = Some(
                        Message::deserialize_from_str(&json_str)
                            .map_err(ExecutionError::JSONDeserialize),
                    );
                    //
                    // // Alternative
                    //
                    // // Spawn and save a tracked deserialized future, then go poll it
                    // *this.deserialize_fut =
                    //     Some(Box::pin(this.compute_executor.spawn(async move {
                    //         Message::deserialize_from_str(&json_str)
                    //             .map_err(ExecutionError::JSONDeserialize)
                    //     })));
                    continue;
                }
                // We successfully got an unexpected websocket message type that was not text
                Poll::Ready(Some(Ok(unexpected))) => {
                    return Poll::Ready(Some(Err(ExecutionError::UnexpectedMessageType(
                        unexpected,
                    ))));
                }
                // We failed to get the next websocket message
                Poll::Ready(Some(Err(err))) => {
                    return Poll::Ready(Some(Err(ExecutionError::WSReadIO(err))));
                }
                // We see the end of the websocket stream, but finish was never sent
                Poll::Ready(None) => {
                    return Poll::Ready(Some(Err(ExecutionError::WSClosedBeforeFinish)));
                }
                // Not ready, so...not ready!
                Poll::Pending => return Poll::Pending,
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
