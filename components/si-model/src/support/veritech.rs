use futures::{FutureExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use si_data::{EventLogFS, EventLogFSError};

use crate::entity::{InferPropertiesRequest, InferPropertiesResponse};

#[derive(Error, Debug)]
pub enum VeritechError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("bad automerge response")]
    BadAutomerge,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("binary websocket data is not supported")]
    Binary,
    #[error("websocket error: {0}")]
    WebSocket(String),
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
    //   #[error("event: {0}")]
    //   Event(#[from] EventError),
    //   #[error("eventLog: {0}")]
    //   EventLog(#[from] EventLogError),
    #[error("missing eventLog: {0}")]
    MissingEventLog(u64),
    #[error("no reply from veritech")]
    NoReply,
    #[error("event log fs error: {0}")]
    EventLogFS(#[from] EventLogFSError),
}

pub type VeritechResult<T> = Result<T, VeritechError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolLog {
    pub fake_id: u64,
    //pub level: EventLogLevel,
    pub message: String,
    pub payload: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolOutputLine {
    pub fake_id: u64,
    pub event_log_id: u64,
    //pub stream: OutputLineStream,
    pub line: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Protocol<REP> {
    pub reply: Option<REP>,
    pub log: Option<ProtocolLog>,
    pub output_line: Option<ProtocolOutputLine>,
}

#[allow(dead_code)]
impl<REP> Protocol<REP> {
    fn has_reply(&self) -> bool {
        self.reply.is_some()
    }

    fn get_reply(self) -> REP {
        self.reply
            .expect("cannot get reply; should've checked with has_reply")
    }

    fn has_log(&self) -> bool {
        self.log.is_some()
    }

    fn get_log(self) -> ProtocolLog {
        self.log
            .expect("cannot get log; should've checked with has_log")
    }

    fn has_output_line(&self) -> bool {
        self.output_line.is_some()
    }

    fn get_output_line(self) -> ProtocolOutputLine {
        self.output_line
            .expect("cannot get output_line; should've checked with has_output_line")
    }
}

#[derive(Clone, Debug)]
pub struct Veritech {
    ws_url: String,
    http_url: String,
    client: reqwest::Client,
    event_log_fs: EventLogFS,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum VeritechMessage<PROGRESS: std::fmt::Debug> {
    Protocol(PROGRESS),
}

impl Veritech {
    pub fn new(settings: &si_settings::Veritech, event_log_fs: EventLogFS) -> Self {
        let ws_url = settings.ws_url.clone();
        let http_url = settings.http_url.clone();
        let client = reqwest::Client::new();
        Self {
            ws_url,
            http_url,
            event_log_fs,
            client,
        }
    }

    pub async fn infer_properties(
        &self,
        request: InferPropertiesRequest,
    ) -> VeritechResult<InferPropertiesResponse> {
        let response: InferPropertiesResponse = self.send_sync("inferProperties", request).await?;
        Ok(response)
    }

    pub async fn send_sync<REQ, REP>(
        &self,
        path: impl AsRef<str>,
        request: REQ,
    ) -> VeritechResult<REP>
    where
        REQ: Serialize,
        REP: DeserializeOwned + std::fmt::Debug,
    {
        let path = path.as_ref();
        let full_url = format!("{}/{}", self.http_url, path);
        let res = self
            .client
            .post(&full_url)
            .json(&serde_json::json!(&request))
            .send()
            .await?;
        let response: REP = res.json().await?;
        Ok(response)
    }

    pub async fn send_async<REQ, PROTOCOL>(
        &self,
        path: impl AsRef<str>,
        request: REQ,
        progress_tx: tokio::sync::mpsc::UnboundedSender<PROTOCOL>,
    ) -> VeritechResult<()>
    where
        REQ: Serialize,
        PROTOCOL: 'static + DeserializeOwned + std::fmt::Debug + Send,
    {
        let (ws_stream, _) =
            tokio_tungstenite::connect_async(format!("{}/{}", &self.ws_url, path.as_ref()))
                .await
                .expect("cannot connect to websocket");

        let (ws_tx, ws_rx) = tokio::sync::mpsc::unbounded_channel();

        let (ws_write, mut ws_read) = ws_stream.split();

        tokio::task::spawn(
            tokio_stream::wrappers::UnboundedReceiverStream::new(ws_rx)
                .forward(ws_write)
                .map(move |result| {
                    if let Err(ref err) = result {
                        // Doesn't look like `warp::Error` can deref the inner error, which is rather sad. The
                        // "Connection closed normall" 'error' appears to be safe and benign, so we'll ignore
                        // this one and warn on all others
                        match err.to_string().as_ref() {
                            "Connection closed normally" => {
                                dbg!("ws client send closed normally; err={:?}", err);
                            }
                            _ => {
                                dbg!("ws client send error; err={}", err);
                            }
                        }
                        result
                    } else {
                        result
                    }
                }),
        );

        let req = serde_json::to_string(&request).expect("failed making request json");

        tokio::spawn(async move {
            match ws_tx.send(Ok(tungstenite::protocol::Message::text(req))) {
                Ok(_) => {}
                Err(e) => {
                    dbg!(&e);
                }
            };
            while let Some(message_result) = ws_read.next().await {
                match message_result {
                    Ok(tungstenite::protocol::Message::Text(data)) => {
                        let reply: VeritechMessage<PROTOCOL> = match serde_json::from_str(&data) {
                            Ok(data) => data,
                            Err(e) => {
                                dbg!("failed to deserialize: {:?}", e);
                                continue;
                            }
                        };
                        dbg!("---- a reply ----");
                        dbg!(&reply);
                        match reply {
                            VeritechMessage::Protocol(progress) => {
                                match progress_tx.send(progress) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        dbg!("cannot send progress down channel; abort!: {:?}", &e);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Ok(tungstenite::protocol::Message::Binary(_)) => {
                        dbg!("received binary message; we only accept text");
                        return;
                    }
                    Ok(tungstenite::protocol::Message::Close(data)) => match data {
                        Some(frame) => match frame.code {
                            tungstenite::protocol::frame::coding::CloseCode::Normal => {
                                dbg!("closed socket normally");
                            }
                            err => {
                                dbg!("request failed; err={:?}", err);
                                dbg!(&frame);
                                return;
                            }
                        },
                        None => {
                            dbg!("websocket closed for unknown reasons");
                            return;
                        }
                    },
                    Ok(tungstenite::protocol::Message::Ping(data)) => {
                        dbg!("ping; data={:?}", data);
                    }
                    Ok(tungstenite::protocol::Message::Pong(data)) => {
                        dbg!("pong; data={:?}", data);
                    }
                    Err(e) => {
                        dbg!("received an error: {:?}", &e);
                        return;
                    }
                }
            }
        });
        Ok(())
    }
}
