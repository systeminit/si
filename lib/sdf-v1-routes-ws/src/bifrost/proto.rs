use std::{
    error::Error,
    sync::Arc,
};

use axum::extract::ws::{
    self,
    WebSocket,
};
use dal::{
    ChangeSetId,
    Ulid,
    WorkspacePk,
};
use frigg::{
    FriggError,
    FriggStore,
};
use nats_multiplexer_client::{
    MultiplexerClient,
    MultiplexerClientError,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::{
    NatsClient,
    Subject,
};
use si_events::workspace_snapshot::EntityKind;
use si_frontend_mv_types::object::{
    FrontendObject,
    patch::{
        DATA_CACHE_SUBJECT_PREFIX,
        UpdateMeta,
    },
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    Mutex,
    broadcast,
};
use tokio_tungstenite::tungstenite;
use tokio_util::sync::CancellationToken;

type Result<T> = std::result::Result<T, BifrostError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum BifrostError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("Broadcast channel receive error: {0}")]
    BroadcastReceive(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("Frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("Multiplexer client error: {0}")]
    MultiplexerClient(#[from] MultiplexerClientError),
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Token cancellation error: {0}")]
    TokenCancellation(#[from] tokio::task::JoinError),
    #[error("TryLock error: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("Error closing websocket: {0}")]
    WsClose(#[source] axum::Error),
    #[error("Error sending websocket message: {0}")]
    WsSendIo(#[source] axum::Error),
}

pub fn run(
    nats: NatsClient,
    frigg: FriggStore,
    workspace_pk: WorkspacePk,
    token: CancellationToken,
) -> Bifrost {
    Bifrost {
        nats,
        frigg,
        workspace_pk,
        token,
    }
}

#[derive(Debug)]
pub struct Bifrost {
    nats: NatsClient,
    frigg: FriggStore,
    workspace_pk: WorkspacePk,
    token: CancellationToken,
}

impl Bifrost {
    pub async fn start(
        self,
        bifrost_multiplexer_client: Arc<Mutex<MultiplexerClient>>,
    ) -> Result<BifrostStarted> {
        // subject is wildcarded, but as of this moment could be either patch_message or index_update
        // for now, just pass everything right along
        let subject = Subject::from(format!(
            "{}.workspace_id.{}.*",
            DATA_CACHE_SUBJECT_PREFIX, self.workspace_pk
        ));
        let receiver = bifrost_multiplexer_client
            .try_lock()?
            .receiver(subject)
            .await?;

        Ok(BifrostStarted {
            workspace_pk: self.workspace_pk,
            _nats: self.nats.clone(),
            frigg: self.frigg.clone(),
            receiver,
            token: self.token,
        })
    }
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum BifrostEventRequest {
    #[serde(rename_all = "camelCase")]
    FetchMV {
        change_set_id: ChangeSetId,
        kind: EntityKind,
        id: String,
        object_checksum: String,
        request_ulid: Option<Ulid>,
    },
}

#[remain::sorted]
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum BifrostEventResponse {
    #[serde(rename_all = "camelCase")]
    FetchMV {
        object: FrontendObject,
        meta: BifrostEventMeta,
    },
}

#[remain::sorted]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BifrostEventMeta {
    pub meta: UpdateMeta,
    pub request_ulid: Option<Ulid>,
}

#[derive(Debug)]
pub struct BifrostStarted {
    workspace_pk: WorkspacePk,
    _nats: NatsClient,
    frigg: FriggStore,
    receiver: broadcast::Receiver<si_data_nats::Message>,
    token: CancellationToken,
}

impl BifrostStarted {
    pub async fn process(mut self, ws: &mut WebSocket) -> Result<BifrostClosing> {
        loop {
            tokio::select! {
                            _ = self.token.cancelled() => {
                                trace!("web socket has received cancellation");
                                let close_frame = ws::CloseFrame {
                                    // Indicates that an endpoint is "going away", such as a server going
                                    // down
                                    code: ws::close_code::AWAY,
                                    // NOTE: reason string must be less than *123* bytes
                                    //
                                    // See: https://en.wikipedia.org/wiki/WebSocket
                                    reason: "endpoint received graceful shutdown".into(),
                                };
                                // Close connection with specific close frame that indicates the server
                                // is going away
                                if let Err(err) = ws.send(ws::Message::Close(Some(close_frame))).await {
                                    // Not much we can or want to do here--we are in the process of
                                    // shutting down
                                    warn!(
                                        error = ?err,
                                        "error while closing websocket connection during graceful shutdown",
                                    );
                                }
                                return Ok(BifrostClosing { ws_is_closed: true });
                            }
                            msg = ws.recv() => {
                                match msg {
                                    Some(Ok(message)) => if let ws::Message::Text(msg) = message {
                                        let event: BifrostEventRequest = match serde_json::from_str(&msg) {
                                            Ok(event) => event,
                                            Err(err) => {

                                                 error!("Unable to deserialize websocket message: {err} {msg}");
                                                    continue;
                                                }
                                        };
                                        match event {
                                            BifrostEventRequest::FetchMV { change_set_id, kind, id, object_checksum, request_ulid } => {
            if let Some(checksum) = match self.frigg.get_index_pointer_value(self.workspace_pk, change_set_id).await? {
                Some((pointer,_revision)) => Some(pointer.index_checksum),
                None => None,
            }{
                let obj = self.frigg
                        .get_object(self.workspace_pk, &kind.to_string(), &id, &object_checksum)
                        .await?;
                    if let Some(object) = obj {
                        let response = BifrostEventResponse::FetchMV{
                    object,
                    meta: BifrostEventMeta { meta: UpdateMeta { workspace_id: self.workspace_pk, change_set_id: Some(change_set_id), to_index_checksum: checksum.to_owned(), from_index_checksum: checksum }, request_ulid },
                };
                let json: serde_json::Value = serde_json::to_value(response)?;
                                                let msg = ws::Message::Text(json.to_string());

                                                 if let Err(err) = ws.send(msg).await {
                                    match err
                                        .source()
                                        .and_then(|err| err.downcast_ref::<tungstenite::Error>())
                                    {
                                            // If the websocket has cleanly closed, we should cleanly finish as
                                            // well--this is not an error condition
                                            Some(tungstenite::Error::ConnectionClosed)
                                            | Some(tungstenite::Error::AlreadyClosed) => {
                                                trace!("websocket has cleanly closed, ending");
                                                return Ok(BifrostClosing { ws_is_closed: true });
                                        },
                                        _ => return Err(BifrostError::WsSendIo(err)),
                                    }
                    }

            }


                                };                                        },
                                        };
                                    },
                                    Some(Err(err)) => return Err(err.into()),
                                    None => return Ok(BifrostClosing { ws_is_closed: true }),
                                }
                            }
                            recv_result = self.receiver.recv() => {
                                let nats_msg =  recv_result?;
                                let msg = ws::Message::Text(String::from_utf8_lossy(nats_msg.payload()).to_string());

                                if let Err(err) = ws.send(msg).await {
                                    match err
                                        .source()
                                        .and_then(|err| err.downcast_ref::<tungstenite::Error>())
                                    {
                                            // If the websocket has cleanly closed, we should cleanly finish as
                                            // well--this is not an error condition
                                            Some(tungstenite::Error::ConnectionClosed)
                                            | Some(tungstenite::Error::AlreadyClosed) => {
                                                trace!("websocket has cleanly closed, ending");
                                                return Ok(BifrostClosing { ws_is_closed: true });
                                        },
                                        _ => return Err(BifrostError::WsSendIo(err)),
                                    }
                                }
                            }
                            else => break,
                        }
        }

        Ok(BifrostClosing {
            ws_is_closed: false,
        })
    }
}

#[derive(Debug)]
pub struct BifrostClosing {
    ws_is_closed: bool,
}

impl BifrostClosing {
    pub async fn finish(self, ws: WebSocket) -> Result<()> {
        if !self.ws_is_closed {
            ws.close().await.map_err(BifrostError::WsClose)?;
        }

        Ok(())
    }
}
