use std::{
    error::{
        self,
        Error as _,
    },
    string::FromUtf8Error,
};

use axum::extract::ws::{
    self,
    WebSocket,
};
use dal::{
    DedicatedExecutor,
    DedicatedExecutorError,
    WorkspacePk,
};
use miniz_oxide::inflate;
use nats_std::header::{
    self,
    value::ContentEncoding,
};
use sdf_core::nats_multiplexer::EddaUpdatesMultiplexerClient;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite;
use tokio_util::sync::CancellationToken;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum BifrostError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("Broadcast channel receive error: {0}")]
    BroadcastReceive(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("compute executor error: {0}")]
    ComputeExecutor(#[from] DedicatedExecutorError),
    #[error("message decompress error: {0}")]
    Decompress(String),
    #[error("edda updates multiplexer client error: {0}")]
    EddaUpdatesMultiplexerClient(#[source] Box<dyn error::Error>),
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("error parsing payload as utf8 string: {0}")]
    PayloadStringParse(#[source] FromUtf8Error),
    #[error("Token cancellation error: {0}")]
    TokenCancellation(#[from] tokio::task::JoinError),
    #[error("Error closing websocket: {0}")]
    WsClose(#[source] axum::Error),
    #[error("Error sending websocket message: {0}")]
    WsSendIo(#[source] axum::Error),
}

type Result<T> = std::result::Result<T, BifrostError>;

type Error = BifrostError;

pub fn run(
    nats: NatsClient,
    compute_executor: DedicatedExecutor,
    workspace_pk: WorkspacePk,
    token: CancellationToken,
) -> Bifrost {
    Bifrost {
        nats,
        compute_executor,
        workspace_pk,
        token,
    }
}

#[derive(Debug)]
pub struct Bifrost {
    nats: NatsClient,
    compute_executor: DedicatedExecutor,
    workspace_pk: WorkspacePk,
    token: CancellationToken,
}

impl Bifrost {
    pub async fn start(
        self,
        bifrost_multiplexer_client: EddaUpdatesMultiplexerClient,
    ) -> Result<BifrostStarted> {
        let receiver = bifrost_multiplexer_client
            .receiver_for_workspace(self.nats.metadata().subject_prefix(), self.workspace_pk)
            .await
            .map_err(Error::EddaUpdatesMultiplexerClient)?;

        Ok(BifrostStarted {
            _workspace_pk: self.workspace_pk,
            _nats: self.nats.clone(),
            compute_executor: self.compute_executor,
            receiver,
            token: self.token,
        })
    }
}

#[derive(Debug)]
pub struct BifrostStarted {
    _workspace_pk: WorkspacePk,
    _nats: NatsClient,
    compute_executor: DedicatedExecutor,
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
                nats_msg_result = self.receiver.recv() => {
                    match nats_msg_result {
                        // We have a message
                        Ok(nats_msg) => {
                            let ws_msg = match self.build_ws_message(nats_msg).await {
                                Ok(ws_msg) => ws_msg,
                                Err(err) => {
                                    warn!(
                                        si.error.message = ?err,
                                        "failed to forward a nats message to web socket; skipping",
                                    );
                                    continue;
                                }
                            };

                            if let Err(err) = ws.send(ws_msg).await {
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
                        // We have a `RecvError`
                        Err(err) => {
                            warn!(
                                si.error.message = ?err,
                                "encountered a recv error on NATS subscription; skipping",
                            );
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

    #[instrument(
        name = "build_ws_message",
        level = "debug",
        skip_all,
        fields(
            bytes.size.compressed = Empty,
            bytes.size.uncompressed = Empty,
        ),
    )]
    async fn build_ws_message(&self, msg: si_data_nats::Message) -> Result<ws::Message> {
        let span = current_span_for_instrument_at!("debug");

        let payload_buf = if header::content_encoding_is(msg.headers(), ContentEncoding::DEFLATE) {
            span.record("bytes.size.compressed", msg.payload().len());
            self.compute_executor
                .spawn(async move {
                    let compressed = msg.into_inner().payload;
                    inflate::decompress_to_vec(&compressed)
                })
                .await?
                .map_err(|decompress_err| Error::Decompress(decompress_err.to_string()))?
        } else if header::content_encoding_is(msg.headers(), ContentEncoding::ZLIB) {
            span.record("bytes.size.compressed", msg.payload().len());
            self.compute_executor
                .spawn(async move {
                    let compressed = msg.into_inner().payload;
                    inflate::decompress_to_vec_zlib(&compressed)
                })
                .await?
                .map_err(|decompress_err| Error::Decompress(decompress_err.to_string()))?
        } else {
            msg.into_inner().payload.into()
        };

        span.record("bytes.size.uncompressed", payload_buf.len());
        let payload_str = String::from_utf8(payload_buf).map_err(Error::PayloadStringParse)?;

        Ok(ws::Message::Text(payload_str))
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
