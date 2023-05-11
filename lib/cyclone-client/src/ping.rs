use futures::StreamExt;
use hyper::client::connect::Connection;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite::Message as WebSocketMessage, WebSocketStream};

pub fn execute<T>(stream: WebSocketStream<T>) -> PingExecution<T> {
    PingExecution { stream }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PingExecutionError {
    #[error("unexpected websocket message after pong was sent: {0}")]
    MessageAfterPong(WebSocketMessage),
    #[error("unexpected websocket message type: {0}")]
    UnexpectedMessageType(WebSocketMessage),
    #[error("unexpected text message other than pong: {0}")]
    UnexpectedText(String),
    #[error("websocket stream is closed, but pong was not sent")]
    WSClosedBeforePong,
    #[error("failed to read websocket message")]
    WSReadIO(#[source] tokio_tungstenite::tungstenite::Error),
}

type Result<T> = std::result::Result<T, PingExecutionError>;

pub struct PingExecution<T> {
    stream: WebSocketStream<T>,
}

impl<T> PingExecution<T>
where
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn start(mut self) -> Result<()> {
        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Text(text))) => {
                if "pong" == text {
                    {}
                } else {
                    return Err(PingExecutionError::UnexpectedText(text));
                }
            }
            Some(Ok(unexpected)) => {
                return Err(PingExecutionError::UnexpectedMessageType(unexpected))
            }
            Some(Err(err)) => return Err(PingExecutionError::WSReadIO(err)),
            None => return Err(PingExecutionError::WSClosedBeforePong),
        };

        match self.stream.next().await {
            Some(Ok(WebSocketMessage::Close(_))) | None => Ok(()),
            Some(Ok(unexpected)) => Err(PingExecutionError::MessageAfterPong(unexpected)),
            Some(Err(err)) => Err(PingExecutionError::WSReadIO(err)),
        }
    }
}
