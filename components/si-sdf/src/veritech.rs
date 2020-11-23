use futures::{FutureExt, SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite;
use tracing::{error, trace, warn};

use std::collections::HashMap;

use crate::data::{Connection, Db};
use crate::models::{
    Entity, Event, EventError, EventKind, EventLog, EventLogError, EventLogLevel, OutputLineStream,
};

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

pub struct Veritech<REQ: Serialize, REP: DeserializeOwned> {
    pub url: String,
    __req: std::marker::PhantomData<REQ>,
    __rep: std::marker::PhantomData<REP>,
}

impl<REQ: Serialize, REP: DeserializeOwned> Veritech<REQ, REP> {
    pub fn new(url: impl Into<String>) -> Veritech<REQ, REP> {
        let url = url.into();
        Veritech {
            url,
            __req: std::marker::PhantomData,
            __rep: std::marker::PhantomData,
        }
    }

    pub async fn send(
        &self,
        db: &Db,
        nats: &Connection,
        request: REQ,
        event: &Event,
    ) -> VeritechResult<Option<REP>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url)
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
                    if reply.has_reply() {
                        message_reply = Some(reply.get_reply());
                    } else if reply.has_log() {
                        let log_msg = reply.get_log();
                        if let Some(mut event_log) = log_cache.get_mut(&log_msg.fake_id) {
                            event_log.level = log_msg.level;
                            event_log.payload = log_msg.payload;
                            event_log.message = log_msg.message;
                            event_log.save(&db, &nats).await?;
                        } else {
                            let event_log = event
                                .log(&db, &nats, log_msg.level, log_msg.message, log_msg.payload)
                                .await?;
                            log_cache.insert(log_msg.fake_id, event_log);
                        }
                    } else if reply.has_output_line() {
                        let output_line_msg = reply.get_output_line();
                        let event_log = log_cache
                            .get(&output_line_msg.event_log_id)
                            .ok_or(VeritechError::MissingEventLog(output_line_msg.event_log_id))?;
                        event_log
                            .output_line(&db, &nats, output_line_msg.stream, output_line_msg.line)
                            .await?;
                    }
                }
                Ok(tungstenite::protocol::Message::Binary(_)) => {
                    warn!("received binary message; we only accept text");
                    return Err(VeritechError::Binary);
                }
                Ok(tungstenite::protocol::Message::Close(data)) => match data {
                    Some(frame) => match frame.code {
                        tungstenite::protocol::frame::coding::CloseCode::Normal => {
                            trace!("closed socket normally");
                        }
                        _ => {
                            warn!(?frame, "request failed");
                            return Err(VeritechError::WebSocket(frame.reason.into()));
                        }
                    },
                    None => {
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
                    warn!(?e, "received an error");
                    return Err(VeritechError::WebSocket(e.to_string()));
                }
            }
        }
        Ok(message_reply)
    }
}
