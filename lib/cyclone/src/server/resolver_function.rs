use std::{io, path::PathBuf, process::Stdio, time::Duration};

use axum::extract::ws::WebSocket;
use bytes_lines_codec::BytesLinesCodec;
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    process::{Child, ChildStdin, ChildStdout, Command},
    time,
};
use tokio_serde::{
    formats::{Json, SymmetricalJson},
    Framed, SymmetricallyFramed,
};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    process::{self, ShutdownError},
    resolver_function::{
        FunctionResult, OutputStream, ResolverFunctionMessage, ResolverFunctionRequest,
        ResultFailure, ResultFailureError, ResultSuccess,
    },
    server::WebSocketMessage,
};

const TX_TIMEOUT_SECS: Duration = Duration::from_secs(2);

pub fn execute(lang_server_path: impl Into<PathBuf>) -> ResolverFunctionExecution {
    ResolverFunctionExecution {
        lang_server_path: lang_server_path.into(),
    }
}

#[derive(Debug, Error)]
pub enum ResolverFunctionError {
    #[error("failed to consume the {0} stream for the child process")]
    ChildIO(&'static str),
    #[error("failed to receive child process message")]
    ChildRecvIO(#[source] io::Error),
    #[error("failed to send child process message")]
    ChildSendIO(#[source] io::Error),
    #[error("failed to spawn child process; program={0}")]
    ChildSpawn(#[source] io::Error, PathBuf),
    #[error(transparent)]
    ChildShutdown(#[from] ShutdownError),
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to close websocket")]
    WSClose(#[source] axum::Error),
    #[error("failed to receive websocket message--stream is closed")]
    WSRecvClosed,
    #[error("failed to receive websocket message")]
    WSRecvIO(#[source] axum::Error),
    #[error("failed to send websocket message")]
    WSSendIO(#[source] axum::Error),
    #[error("send timeout")]
    SendTimeout(#[source] tokio::time::error::Elapsed),
    #[error("unexpected websocket message type: {0:?}")]
    UnexpectedMessageType(WebSocketMessage),
}

type Result<T> = std::result::Result<T, ResolverFunctionError>;

#[derive(Debug)]
pub struct ResolverFunctionExecution {
    lang_server_path: PathBuf,
}

impl ResolverFunctionExecution {
    pub async fn start(self, ws: &mut WebSocket) -> Result<ResolverFunctionServerExecutionStarted> {
        Self::ws_send_start(ws).await?;
        let request = Self::read_request(ws).await?;

        let mut child = Command::new(&self.lang_server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| ResolverFunctionError::ChildSpawn(err, self.lang_server_path.clone()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or(ResolverFunctionError::ChildIO("stdin"))?;
        Self::child_send_function_request(stdin, request).await?;

        let stdout = {
            let stdout = child
                .stdout
                .take()
                .ok_or(ResolverFunctionError::ChildIO("stdout"))?;
            let codec = FramedRead::new(stdout, BytesLinesCodec::new());
            SymmetricallyFramed::new(codec, SymmetricalJson::default())
        };

        Ok(ResolverFunctionServerExecutionStarted { child, stdout })
    }

    async fn read_request(ws: &mut WebSocket) -> Result<ResolverFunctionRequest> {
        let request = match ws.next().await {
            Some(Ok(WebSocketMessage::Text(json_str))) => {
                ResolverFunctionRequest::deserialize_from_str(&json_str)
                    .map_err(ResolverFunctionError::JSONDeserialize)?
            }
            Some(Ok(unexpected)) => {
                return Err(ResolverFunctionError::UnexpectedMessageType(unexpected))
            }
            Some(Err(err)) => return Err(ResolverFunctionError::WSRecvIO(err)),
            None => return Err(ResolverFunctionError::WSRecvClosed),
        };
        Ok(request)
    }

    async fn ws_send_start(ws: &mut WebSocket) -> Result<()> {
        let msg = ResolverFunctionMessage::Start
            .serialize_to_string()
            .map_err(ResolverFunctionError::JSONSerialize)?;

        time::timeout(TX_TIMEOUT_SECS, ws.send(WebSocketMessage::Text(msg)))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::WSSendIO)?;
        Ok(())
    }

    async fn child_send_function_request(
        stdin: ChildStdin,
        request: ResolverFunctionRequest,
    ) -> Result<()> {
        let codec = FramedWrite::new(stdin, BytesLinesCodec::new());
        let mut stdin = SymmetricallyFramed::new(codec, SymmetricalJson::default());

        time::timeout(TX_TIMEOUT_SECS, stdin.send(request))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::ChildSendIO)?;
        time::timeout(TX_TIMEOUT_SECS, stdin.close())
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::ChildSendIO)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ResolverFunctionServerExecutionStarted {
    child: Child,
    stdout: Framed<
        FramedRead<ChildStdout, BytesLinesCodec>,
        LSResolverFunctionMessage,
        LSResolverFunctionMessage,
        Json<LSResolverFunctionMessage, LSResolverFunctionMessage>,
    >,
}

impl ResolverFunctionServerExecutionStarted {
    pub async fn process(
        self,
        ws: &mut WebSocket,
    ) -> Result<ResolverFunctionServerExecutionClosing> {
        let mut stream = self
            .stdout
            .map(|ls_result| match ls_result {
                Ok(ls_msg) => match ls_msg {
                    LSResolverFunctionMessage::Output(output) => Ok(
                        ResolverFunctionMessage::OutputStream(OutputStream::new_from(output)),
                    ),
                    LSResolverFunctionMessage::Result(result) => Ok(
                        ResolverFunctionMessage::FunctionResult(FunctionResult::new_from(result)),
                    ),
                },
                Err(err) => Err(ResolverFunctionError::ChildRecvIO(err)),
            })
            .map(|msg_result: Result<_>| match msg_result {
                Ok(msg) => match msg
                    .serialize_to_string()
                    .map_err(ResolverFunctionError::JSONSerialize)
                {
                    Ok(json_str) => Ok(WebSocketMessage::Text(json_str)),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            });

        while let Some(msg) = stream.try_next().await? {
            ws.send(msg)
                .await
                .map_err(ResolverFunctionError::WSSendIO)?;
        }

        Ok(ResolverFunctionServerExecutionClosing { child: self.child })
    }
}

#[derive(Debug)]
pub struct ResolverFunctionServerExecutionClosing {
    child: Child,
}

impl ResolverFunctionServerExecutionClosing {
    pub async fn finish(mut self, mut ws: WebSocket) -> Result<()> {
        let finished = Self::ws_send_finish(&mut ws).await;
        let closed = Self::ws_close(ws).await;
        let shutdown =
            process::child_shutdown(&mut self.child, Some(process::Signal::SIGTERM), None)
                .await
                .map_err(Into::into);
        drop(self.child);

        match (finished, closed, shutdown) {
            // Everything succeeds, great!
            (Ok(_), Ok(_), Ok(_)) => Ok(()),

            // One of the steps failed, return its error
            (Ok(_), Ok(_), Err(err)) | (Ok(_), Err(err), Ok(_)) | (Err(err), Ok(_), Ok(_)) => {
                Err(err)
            }

            // 2/3 steps errored so warn about the lower priority error and return the highest
            // priority
            (Ok(_), Err(err), Err(shutdown)) => {
                warn!(error = ?shutdown, "failed to shutdown child cleanly");
                Err(err)
            }
            (Err(err), Ok(_), Err(shutdown)) => {
                warn!(error = ?shutdown, "failed to shutdown child cleanly");
                Err(err)
            }
            (Err(err), Err(closed), Ok(_)) => {
                warn!(error = ?closed, "failed to cleanly close websocket");
                Err(err)
            }

            // All steps failed so warn about the lower priorities and return the highest priority
            (Err(err), Err(closed), Err(shutdown)) => {
                warn!(error = ?shutdown, "failed to shutdown child cleanly");
                warn!(error = ?closed, "failed to cleanly close websocket");
                Err(err)
            }
        }
    }

    async fn ws_send_finish(ws: &mut WebSocket) -> Result<()> {
        let msg = ResolverFunctionMessage::Finish
            .serialize_to_string()
            .map_err(ResolverFunctionError::JSONSerialize)?;
        time::timeout(TX_TIMEOUT_SECS, ws.send(WebSocketMessage::Text(msg)))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::WSSendIO)?;

        Ok(())
    }

    async fn ws_close(ws: WebSocket) -> Result<()> {
        ws.close().await.map_err(ResolverFunctionError::WSClose)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "protocol", rename_all = "camelCase")]
enum LSResolverFunctionMessage {
    Output(LSOutput),
    Result(LSResult),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct LSOutput {
    stream: String,
    level: String,
    group: Option<String>,
    data: Option<Value>,
    message: String,
    timestamp: u64,
}

impl OutputStream {
    fn new_from(ls_output: LSOutput) -> Self {
        Self {
            stream: ls_output.stream,
            level: ls_output.level,
            group: ls_output.group,
            data: ls_output.data,
            message: ls_output.message,
            timestamp: ls_output.timestamp,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum LSResult {
    Success(LSSuccess),
    Failure(LSFailure),
}

impl FunctionResult {
    fn new_from(result: LSResult) -> Self {
        match result {
            LSResult::Success(success) => Self::Success(ResultSuccess {
                data: success.data,
                unset: success.unset,
            }),
            LSResult::Failure(failure) => Self::Failure(ResultFailure {
                error: ResultFailureError {
                    message: failure.error.message,
                    name: failure.error.name,
                },
            }),
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct LSSuccess {
    data: Value,
    unset: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct LSFailure {
    error: LSFailureError,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct LSFailureError {
    message: String,
    name: String,
}
