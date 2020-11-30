use futures::{FutureExt, StreamExt};
use serde_json;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use tracing::{trace, warn};

pub use crate::cli::formatter;
pub use crate::cli::formatter::DebugFormatter;
pub use crate::cli::server::{ChangeRun, CliMessage, Command};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("connection failed")]
    Connection,
    #[error("binary data is not allowed")]
    Binary,
    #[error("websocket error: {0}")]
    WebSocket(String),
    #[error("serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("tokio send error: {0}")]
    SendError(
        #[from]
        tokio::sync::mpsc::error::SendError<
            std::result::Result<
                tokio_tungstenite::tungstenite::protocol::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        >,
    ),
}

pub type ClientResult<T> = Result<T, ClientError>;

pub struct Client<F: formatter::Formatter> {
    url: String,
    formatter: F,
}

impl<F: formatter::Formatter> Client<F> {
    pub fn new(url: impl Into<String>, formatter: F) -> Client<F> {
        let url = url.into();

        Client { url, formatter }
    }

    pub async fn command(&mut self, command: Command) -> ClientResult<()> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .expect("cannot connect to websocket");

        let (ws_tx, ws_rx) = tokio::sync::mpsc::unbounded_channel();

        let (ws_write, mut ws_read) = ws_stream.split();
        tokio::task::spawn(ws_rx.forward(ws_write).map(move |result| {
            if let Err(err) = result {
                // Doesn't look like `warp::Error` can deref the inner error, which is rather sad. The
                // "Connection closed normall" 'error' appears to be safe and benign, so we'll ignore
                // this one and warn on all others
                match err.to_string().as_ref() {
                    "Connection closed normally" => {
                        trace!("ws client send closed normally; err={:?}", err)
                    }
                    _ => warn!("ws client send error; err={}", err),
                }
            }
        }));

        let req = serde_json::to_string(&command).expect("failed making request json");

        ws_tx.send(Ok(tungstenite::protocol::Message::text(req)))?;

        while let Some(message_result) = ws_read.next().await {
            match message_result {
                Ok(tungstenite::protocol::Message::Text(data)) => {
                    let cli_message: CliMessage = serde_json::from_str(&data)?;
                    self.formatter.process_message(cli_message)?;
                }
                Ok(tungstenite::protocol::Message::Binary(_)) => {
                    warn!("received binary message; we only accept text");
                    return Err(ClientError::Binary);
                }
                Ok(tungstenite::protocol::Message::Close(data)) => match data {
                    Some(frame) => match frame.code {
                        tungstenite::protocol::frame::coding::CloseCode::Normal => {
                            trace!("closed socket normally");
                            return Ok(());
                        }
                        _ => {
                            warn!(?frame, "request failed");
                            return Err(ClientError::WebSocket(frame.reason.into()));
                        }
                    },
                    None => {
                        warn!("websocket closed for unknown reasons");
                        return Err(ClientError::WebSocket(
                            "websocket closed for unknown reasons".into(),
                        ));
                    }
                },
                Ok(tungstenite::protocol::Message::Ping(data)) => {
                    dbg!("ping");
                    dbg!(data);
                }
                Ok(tungstenite::protocol::Message::Pong(data)) => {
                    dbg!("pong");
                    dbg!(data);
                }
                Err(e) => {
                    warn!(?e, "received an error");
                    return Err(ClientError::WebSocket(e.to_string()));
                }
            }
        }

        Ok(())
    }
}
