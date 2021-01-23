use futures::{FutureExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use tracing::{error, trace, warn};

use std::collections::HashMap;

use crate::data::{EventLogFS, EventLogFSError, NatsConn, PgPool};
use crate::models::{Event, EventError, EventLog, EventLogError, EventLogLevel, OutputLineStream};

#[derive(Error, Debug)]
pub enum VeritechError {
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
    #[error("event: {0}")]
    Event(#[from] EventError),
    #[error("eventLog: {0}")]
    EventLog(#[from] EventLogError),
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
    pub level: EventLogLevel,
    pub message: String,
    pub payload: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolOutputLine {
    pub fake_id: u64,
    pub event_log_id: u64,
    pub stream: OutputLineStream,
    pub line: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Protocol<REP> {
    pub reply: Option<REP>,
    pub log: Option<ProtocolLog>,
    pub output_line: Option<ProtocolOutputLine>,
}

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
    url: String,
    event_log_fs: EventLogFS,
}

impl Veritech {
    pub fn new(settings: &si_settings::Veritech, event_log_fs: EventLogFS) -> Self {
        let url = settings.url.clone();
        Self { url, event_log_fs }
    }

    pub async fn send<REQ, REP>(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        path: impl AsRef<str>,
        request: REQ,
        event: &Event,
    ) -> VeritechResult<Option<REP>>
    where
        REQ: Serialize,
        REP: DeserializeOwned + std::fmt::Debug,
    {
        let (ws_stream, _) =
            tokio_tungstenite::connect_async(format!("{}{}", &self.url, path.as_ref()))
                .await
                .expect("cannot connect to websocket");

        let (ws_tx, ws_rx) = tokio::sync::mpsc::unbounded_channel();

        //ws_stream.send_all(stream)
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
        let req = serde_json::to_string(&request).expect("failed making request json");

        ws_tx.send(Ok(tungstenite::protocol::Message::text(req)))?;

        let mut log_cache: HashMap<u64, EventLog> = HashMap::new();
        let mut message_reply: Option<REP> = None;
        while let Some(message_result) = ws_read.next().await {
            match message_result {
                Ok(tungstenite::protocol::Message::Text(data)) => {
                    let reply: Protocol<REP> = serde_json::from_str(&data)?;
                    dbg!("**** websocket message ****");
                    dbg!(&reply);
                    if reply.has_reply() {
                        dbg!("*** has a reply ***");
                        message_reply = Some(reply.get_reply());
                    } else if reply.has_log() {
                        let log_msg = reply.get_log();
                        if let Some(mut event_log) = log_cache.get_mut(&log_msg.fake_id) {
                            event_log.level = log_msg.level;
                            event_log.payload = log_msg.payload;
                            event_log.message = log_msg.message;
                            event_log.save(&pg, &nats_conn).await?;
                        } else {
                            let event_log = event
                                .log(
                                    &pg,
                                    &nats_conn,
                                    log_msg.level,
                                    log_msg.message,
                                    log_msg.payload,
                                )
                                .await?;
                            log_cache.insert(log_msg.fake_id, event_log);
                        }
                    } else if reply.has_output_line() {
                        let output_line_msg = reply.get_output_line();
                        let event_log = log_cache
                            .get_mut(&output_line_msg.event_log_id)
                            .ok_or(VeritechError::MissingEventLog(output_line_msg.event_log_id))?;
                        event_log
                            .output_line(
                                &pg,
                                &nats_conn,
                                &self.event_log_fs,
                                output_line_msg.stream,
                                output_line_msg.line,
                                false,
                            )
                            .await?;
                    }
                }
                Ok(tungstenite::protocol::Message::Binary(_)) => {
                    dbg!("received binary message; we only accept text");
                    return Err(VeritechError::Binary);
                }
                Ok(tungstenite::protocol::Message::Close(data)) => match data {
                    Some(frame) => match frame.code {
                        tungstenite::protocol::frame::coding::CloseCode::Normal => {
                            dbg!("normal reason for closing");
                            trace!("closed socket normally");
                        }
                        e => {
                            dbg!("closing errork");
                            dbg!(&e);
                            warn!(?frame, "request failed");
                            return Err(VeritechError::WebSocket(frame.reason.into()));
                        }
                    },
                    None => {
                        dbg!("unknown reason for closing");
                        warn!("websocket closed for unknown reasons");
                        return Err(VeritechError::WebSocket(
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
                    dbg!("*** received an error ***");
                    dbg!(&e);
                    warn!(?e, "received an error");
                    return Err(VeritechError::WebSocket(e.to_string()));
                }
            }
        }
        Ok(message_reply)
    }
}
