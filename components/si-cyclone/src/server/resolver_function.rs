use std::{io, path::PathBuf, process::Stdio, time::Duration};

use axum::extract::ws::WebSocket;
use bytes_lines_codec::BytesLinesCodec;
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
use tracing::warn;

use crate::{
    resolver_function::{
        FunctionResult, OutputStream, ResolverFunctionMessage, ResolverFunctionRequest,
        ResultFailure, ResultFailureError, ResultSuccess,
    },
    server::WebSocketMessage,
};

const CHILD_WAIT_TIMEOUT_SECS: Duration = Duration::from_secs(10);
const TX_TIMEOUT_SECS: Duration = Duration::from_secs(2);

pub fn execute(
    socket: WebSocket,
    lang_server_path: impl Into<PathBuf>,
) -> ResolverFunctionExecution {
    ResolverFunctionExecution {
        ws: socket,
        lang_server_path: lang_server_path.into(),
    }
}

#[derive(Debug, Error)]
pub enum ResolverFunctionError {
    #[error("failed to consume the {0} stream for the child process")]
    ChildIO(&'static str),
    #[error("failed to send child process message")]
    ChildSendIO(#[source] io::Error),
    #[error("failed to spawn child process; program={0}")]
    ChildSpawn(#[source] io::Error, PathBuf),
    #[error("failed to wait on child process")]
    ChildWait(#[source] io::Error),
    #[error("failed to deserialize json message")]
    JSONDeserialize(#[source] serde_json::Error),
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to close websocket")]
    WSClose(#[source] axum::Error),
    #[error("failed to receive websocket message")]
    WSRecvIO(#[source] axum::Error),
    #[error("failed to send websocket message")]
    WSSendIO(#[source] axum::Error),
    #[error("send timeout")]
    SendTimeout(#[source] tokio::time::error::Elapsed),
}

#[derive(Debug)]
pub struct ResolverFunctionExecution {
    ws: WebSocket,
    lang_server_path: PathBuf,
}

impl ResolverFunctionExecution {
    pub async fn start(
        mut self,
    ) -> Result<ResolverFunctionServerExecutionStarted, ResolverFunctionError> {
        self.ws_send_start().await?;
        let request = self.read_request().await?;

        let mut child = Command::new(&self.lang_server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // .stderr(Stdio::piped())
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
        // let stderr = {
        //     let stderr = child
        //         .stderr
        //         .take()
        //         .ok_or(ResolverFunctionError::ChildIO("stderr"))?;
        //     let codec = FramedRead::new(stderr, BytesLinesCodec::new());
        //     SymmetricallyFramed::new(codec, SymmetricalJson::default())
        // };

        Ok(ResolverFunctionServerExecutionStarted {
            ws: self.ws,
            child,
            stdout,
            // stderr,
        })
    }

    async fn read_request(&mut self) -> Result<ResolverFunctionRequest, ResolverFunctionError> {
        let request = match self.ws.next().await {
            Some(Ok(WebSocketMessage::Text(request_json))) => {
                let msg: ResolverFunctionRequest = serde_json::from_str(&request_json)
                    .map_err(ResolverFunctionError::JSONDeserialize)?;
                msg
            }
            Some(Ok(unexpected)) => panic!("unexpected websocket message type: {:?}", unexpected),
            Some(Err(err)) => panic!("websocket errored: {:?}", err),
            None => panic!(),
        };
        Ok(request)
    }

    async fn ws_send_start(&mut self) -> Result<(), ResolverFunctionError> {
        let msg = WebSocketMessage::Text(
            serde_json::to_string(&ResolverFunctionMessage::Start)
                .map_err(ResolverFunctionError::JSONSerialize)?,
        );

        time::timeout(TX_TIMEOUT_SECS, self.ws.send(msg))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::WSSendIO)?;
        Ok(())
    }

    async fn child_send_function_request(
        stdin: ChildStdin,
        request: ResolverFunctionRequest,
    ) -> Result<(), ResolverFunctionError> {
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
    ws: WebSocket,
    child: Child,
    stdout: Framed<
        FramedRead<ChildStdout, BytesLinesCodec>,
        LSResolverFunctionMessage,
        LSResolverFunctionMessage,
        Json<LSResolverFunctionMessage, LSResolverFunctionMessage>,
    >,
    // stderr: Framed<FramedRead<ChildStderr, BytesLinesCodec>, Value, Value, Json<Value, Value>>,
}

impl ResolverFunctionServerExecutionStarted {
    pub async fn process(
        mut self,
    ) -> Result<ResolverFunctionServerExecutionClosing, ResolverFunctionError> {
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
                Err(err) => panic!("failed to read a message from child: {:?}", err),
            })
            .map(
                |msg_result: Result<_, ResolverFunctionError>| match msg_result {
                    Ok(msg) => match serde_json::to_string(&msg)
                        .map_err(ResolverFunctionError::JSONSerialize)
                    {
                        Ok(json_str) => Ok(WebSocketMessage::Text(json_str)),
                        Err(err) => Err(err),
                    },
                    Err(err) => panic!("things are going bad, yo: {:?}", err),
                },
            );

        while let Some(msg) = stream.try_next().await? {
            self.ws
                .send(msg)
                .await
                .map_err(ResolverFunctionError::WSSendIO)?;
        }

        Ok(ResolverFunctionServerExecutionClosing {
            ws: self.ws,
            child: self.child,
        })
    }
}

#[derive(Debug)]
pub struct ResolverFunctionServerExecutionClosing {
    ws: WebSocket,
    child: Child,
}

impl ResolverFunctionServerExecutionClosing {
    pub async fn finish(mut self) -> Result<(), ResolverFunctionError> {
        self.ws_send_finish().await?;
        Self::ws_close(self.ws).await?;
        Self::child_shutdown(self.child).await?;

        Ok(())
    }

    async fn ws_send_finish(&mut self) -> Result<(), ResolverFunctionError> {
        let msg = WebSocketMessage::Text(
            serde_json::to_string(&ResolverFunctionMessage::Finish)
                .map_err(ResolverFunctionError::JSONSerialize)?,
        );
        time::timeout(TX_TIMEOUT_SECS, self.ws.send(msg))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::WSSendIO)?;
        Ok(())
    }

    async fn ws_close(ws: WebSocket) -> Result<(), ResolverFunctionError> {
        ws.close().await.map_err(ResolverFunctionError::WSClose)
    }

    async fn child_shutdown(mut child: Child) -> Result<(), ResolverFunctionError> {
        Ok(
            match time::timeout(CHILD_WAIT_TIMEOUT_SECS, child.wait()).await {
                Ok(wait_result) => {
                    let exit_status = wait_result.map_err(ResolverFunctionError::ChildWait)?;
                    warn!("child process had a nonzero exit; code={}", exit_status);
                }
                Err(_elapsed) => {
                    if let Ok(_) = child.start_kill() {
                        let exit_status = child
                            .wait()
                            .await
                            .map_err(ResolverFunctionError::ChildWait)?;
                        warn!("child process had a nonzero exit; code={}", exit_status);
                    }
                }
            },
        )
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
