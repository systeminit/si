use std::{
    path::PathBuf,
    pin::Pin,
    process::Stdio,
    task::{Context, Poll},
    time::Duration,
};

use axum::extract::ws::WebSocket;
use bytes_lines_codec::BytesLinesCodec;
use futures::{Future, SinkExt, Stream, StreamExt};
use serde_json::Value;
use thiserror::Error;
use tokio::{
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    time,
};
use tokio_serde::{
    formats::{Json, SymmetricalJson},
    Framed, SymmetricallyFramed,
};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    resolver_function::{ResolverFunctionMessage, ResolverFunctionRequest},
    server::WebSocketMessage,
};

// const RX_TIMEOUT_SECS: Duration = Duration::from_secs(2);
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
    #[error("failed to spawn child process; program={0}")]
    ChildSpawn(#[source] std::io::Error, PathBuf),
    #[error("failed to serialize json message")]
    JSONSerialize(#[source] serde_json::Error),
    #[error("failed to send websocket message")]
    SendIO(#[source] axum::Error),
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

        let mut child = Command::new(&self.lang_server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| ResolverFunctionError::ChildSpawn(err, self.lang_server_path.clone()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or(ResolverFunctionError::ChildIO("stdin"))?;
        Self::child_send_function_request(stdin).await?;

        let stdout = {
            let stdout = child
                .stdout
                .take()
                .ok_or(ResolverFunctionError::ChildIO("stdout"))?;
            let codec = FramedRead::new(stdout, BytesLinesCodec::new());
            SymmetricallyFramed::new(codec, SymmetricalJson::default())
        };
        let stderr = {
            let stderr = child
                .stderr
                .take()
                .ok_or(ResolverFunctionError::ChildIO("stderr"))?;
            let codec = FramedRead::new(stderr, BytesLinesCodec::new());
            SymmetricallyFramed::new(codec, SymmetricalJson::default())
        };

        Ok(ResolverFunctionServerExecutionStarted {
            ws: self.ws,
            child,
            stdout,
            stderr,
        })
    }

    async fn ws_send_start(&mut self) -> Result<(), ResolverFunctionError> {
        let msg = WebSocketMessage::Text(
            serde_json::to_string(&ResolverFunctionMessage::Start)
                .map_err(ResolverFunctionError::JSONSerialize)?,
        );
        time::timeout(TX_TIMEOUT_SECS, self.ws.send(msg))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::SendIO)?;
        Ok(())
    }

    async fn child_send_function_request(stdin: ChildStdin) -> Result<(), ResolverFunctionError> {
        let codec = FramedWrite::new(stdin, BytesLinesCodec::new());
        let mut stdin = SymmetricallyFramed::new(codec, SymmetricalJson::default());
        stdin.send(ResolverFunctionRequest {}).await.unwrap();
        stdin.close().await.unwrap();
        Ok(())
    }
}

#[derive(Debug)]
pub struct ResolverFunctionServerExecutionStarted {
    ws: WebSocket,
    child: Child,
    stdout: Framed<
        FramedRead<ChildStdout, BytesLinesCodec>,
        ResolverFunctionMessage,
        ResolverFunctionMessage,
        Json<ResolverFunctionMessage, ResolverFunctionMessage>,
    >,
    stderr: Framed<FramedRead<ChildStderr, BytesLinesCodec>, Value, Value, Json<Value, Value>>,
}

impl ResolverFunctionServerExecutionStarted {
    pub async fn finish(mut self) -> Result<(), ResolverFunctionError> {
        self.ws_send_finish().await
    }

    async fn ws_send_finish(&mut self) -> Result<(), ResolverFunctionError> {
        let msg = WebSocketMessage::Text(
            serde_json::to_string(&ResolverFunctionMessage::Finish)
                .map_err(ResolverFunctionError::JSONSerialize)?,
        );
        time::timeout(TX_TIMEOUT_SECS, self.ws.send(msg))
            .await
            .map_err(ResolverFunctionError::SendTimeout)?
            .map_err(ResolverFunctionError::SendIO)?;
        Ok(())
    }
}

impl Stream for ResolverFunctionServerExecutionStarted {
    type Item = Result<ResolverFunctionMessage, ResolverFunctionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stdout.next()).poll(cx) {
            Poll::Ready(a) => todo!("cool: {:?}", a),
            Poll::Pending => Poll::Pending,
        }
    }
}
