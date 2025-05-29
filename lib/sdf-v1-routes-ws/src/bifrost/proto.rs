use std::{
    error::Error,
    sync::Arc,
};

use axum::extract::ws::{
    self,
    WebSocket,
};
use dal::WorkspacePk;
use nats_multiplexer_client::{
    MultiplexerClient,
    MultiplexerClientError,
};
use si_data_nats::{
    NatsClient,
    Subject,
};
use si_frontend_mv_types::object::patch::DATA_CACHE_SUBJECT_PREFIX;
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
    #[error("Multiplexer client error: {0}")]
    MultiplexerClient(#[from] MultiplexerClientError),
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("Token cancellation error: {0}")]
    TokenCancellation(#[from] tokio::task::JoinError),
    #[error("TryLock error: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("Error closing websocket: {0}")]
    WsClose(#[source] axum::Error),
    #[error("Error sending websocket message: {0}")]
    WsSendIo(#[source] axum::Error),
}

pub fn run(nats: NatsClient, workspace_pk: WorkspacePk, token: CancellationToken) -> Bifrost {
    Bifrost {
        nats,
        workspace_pk,
        token,
    }
}

#[derive(Debug)]
pub struct Bifrost {
    nats: NatsClient,
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
            _workspace_pk: self.workspace_pk,
            _nats: self.nats.clone(),
            receiver,
            token: self.token,
        })
    }
}

#[derive(Debug)]
pub struct BifrostStarted {
    _workspace_pk: WorkspacePk,
    _nats: NatsClient,
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
                        Some(Ok(_)) => {
                            // We don't support any incoming commands over this websocket yet, but
                            // when we do, this is where we'd handle dispatch for them.
                            continue;

                        }
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
