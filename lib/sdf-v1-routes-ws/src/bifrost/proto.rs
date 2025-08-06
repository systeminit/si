use std::{
    error::{
        self,
        Error as _,
    },
    fmt::{
        Debug,
        Formatter,
    },
    string::FromUtf8Error,
    sync::Arc,
};

use axum::extract::ws::{
    self,
    WebSocket,
};
use dal::{
    ChangeSetId,
    DedicatedExecutor,
    DedicatedExecutorError,
    WorkspacePk,
};
use frigg::FriggStore;
use miniz_oxide::inflate;
use nats_multiplexer_client::MultiplexerClientError;
use nats_std::header::{
    self,
    value::ContentEncoding,
};
use sdf_core::{
    index::FrontEndObjectMeta,
    nats_multiplexer::EddaUpdatesMultiplexerClient,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::{
    ConnectionMetadata,
    Message,
};
use si_frontend_types::FrontEndObjectRequest;
use task::{
    BifrostFriggReadsTask,
    BifrostFriggReadsTaskHandle,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::{
    StreamExt,
    adapters::Merge,
    wrappers::BroadcastStream,
};
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
    #[error("frigg reads task recv error: channel is empty and closed")]
    FriggReadsTaskRecv,
    #[error("error serialize frigg reads response: {0}")]
    FriggReadsTaskResponseSerialize(#[source] serde_json::Error),
    #[error("frigg reads task send error: channel is closed or rx dropped")]
    FriggReadsTaskSend,
    #[error("Multiplexer client error: {0}")]
    MultiplexerClient(#[from] MultiplexerClientError),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsFrontEndOjbectRequest {
    pub ws_request_id: Option<String>,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    #[serde(flatten)]
    pub request: FrontEndObjectRequest,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "result")]
pub enum WsFrontEndObjectResponse {
    Ok {
        ws_request_id: Option<String>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        #[serde(flatten)]
        response: FrontEndObjectMeta,
    },
    Err {
        ws_request_id: Option<String>,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        error: String,
        #[serde(flatten)]
        request: FrontEndObjectRequest,
    },
}

pub fn run(
    metadata: Arc<ConnectionMetadata>,
    frigg: FriggStore,
    compute_executor: DedicatedExecutor,
    workspace_id: WorkspacePk,
    token: CancellationToken,
) -> Bifrost {
    Bifrost {
        metadata,
        frigg,
        compute_executor,
        workspace_id,
        token,
    }
}

#[derive(Debug)]
pub struct Bifrost {
    metadata: Arc<ConnectionMetadata>,
    frigg: FriggStore,
    compute_executor: DedicatedExecutor,
    workspace_id: WorkspacePk,
    token: CancellationToken,
}

impl Bifrost {
    pub async fn start(
        self,
        bifrost_multiplexer_client: EddaUpdatesMultiplexerClient,
    ) -> Result<BifrostStarted> {
        let nats_workspace_messages = bifrost_multiplexer_client
            .messages_for_workspace(self.metadata.subject_prefix(), self.workspace_id)
            .await
            .map_err(Error::EddaUpdatesMultiplexerClient)?;

        let nats_deployment_messages = bifrost_multiplexer_client
            .messages_for_deployment(self.metadata.subject_prefix())
            .await
            .map_err(Error::EddaUpdatesMultiplexerClient)?;

        let workspace_messages =
            tokio_stream::wrappers::BroadcastStream::new(nats_workspace_messages);
        let deployment_messages =
            tokio_stream::wrappers::BroadcastStream::new(nats_deployment_messages);
        let nats_messages = workspace_messages.merge(deployment_messages);

        let (requests_tx, requests_rx) = mpsc::channel(256);
        let (responses_tx, responses_rx) = mpsc::channel(128);

        let handle = {
            let task_token = self.token.child_token();

            // We will await shutdown of this task via its [`JoinHandle`], hence no need for a
            // [`TaskTracker`].
            let join_handle = tokio::spawn(
                BifrostFriggReadsTask::create(
                    self.frigg,
                    requests_rx,
                    responses_tx,
                    task_token.clone(),
                )
                .run(),
            );

            BifrostFriggReadsTaskHandle::new(join_handle, task_token)
        };

        Ok(BifrostStarted {
            compute_executor: self.compute_executor,
            nats_messages,
            requests_tx,
            responses_rx,
            handle,
            token: self.token,
        })
    }
}

pub struct BifrostStarted {
    compute_executor: DedicatedExecutor,
    nats_messages: Merge<BroadcastStream<Message>, BroadcastStream<Message>>,
    requests_tx: mpsc::Sender<WsFrontEndOjbectRequest>,
    responses_rx: mpsc::Receiver<WsFrontEndObjectResponse>,
    handle: BifrostFriggReadsTaskHandle,
    token: CancellationToken,
}

impl Debug for BifrostStarted {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BifrostStarted")
            .field("compute_executor", &self.compute_executor)
            .field("requests_tx", &self.requests_tx)
            .field("responses_rx", &self.responses_rx)
            .field("handle", &self.handle)
            .field("token", &self.token)
            .finish_non_exhaustive()
    }
}

impl BifrostStarted {
    pub async fn process(mut self, ws_client: &mut WebSocket) -> Result<BifrostClosing> {
        loop {
            tokio::select! {
                // Cancellation token has fired, time to shut down
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
                    if let Err(err) = ws_client.send(ws::Message::Close(Some(close_frame))).await {
                        // Not much we can or want to do here--we are in the process of
                        // shutting down
                        warn!(
                            error = ?err,
                            "error while closing websocket connection during graceful shutdown",
                        );
                    }
                    return Ok(BifrostClosing {
                        ws_is_closed: true,
                        handle: self.handle,
                    });
                }
                // Maybe a message from web socket client
                maybe_ws_client_message = ws_client.recv() => {
                    match maybe_ws_client_message {
                        // Received web socket text message
                        Some(Ok(ws::Message::Text(payload))) => {
                            let request: WsFrontEndOjbectRequest =
                                match serde_json::from_str(&payload) {
                                    // Deserialize successful
                                    Ok(r) => r,
                                    // Error while deserializing
                                    Err(err) => {
                                        warn!(
                                            si.error.message = ?err,
                                            "client request failed to deserialize; skipping",
                                        );
                                        continue;
                                    }
                                };

                            self.requests_tx
                                .send(request)
                                .await
                                .map_err(|_| BifrostError::FriggReadsTaskSend)?;
                        }
                        // Received a ping (library automatically deals with replies)
                        Some(Ok(ws::Message::Ping(payload))) => {
                            trace!(
                                ws.client.ping.message = String::from_utf8_lossy(&payload).as_ref(),
                                "read web socket client ping message; skipping",
                            );
                            continue;
                        }
                        // Received a ping (library automatically deals with replies)
                        Some(Ok(ws::Message::Pong(payload))) => {
                            trace!(
                                ws.client.pong.message = String::from_utf8_lossy(&payload).as_ref(),
                                "read web socket client pong message; skipping",
                            );
                            continue;
                        }
                        // Received a close message from the client
                        Some(Ok(ws::Message::Close(maybe_close_frame))) => {
                            debug!(
                                ws.client.close.frame = ?maybe_close_frame,
                                "read web socket client close message; shutting down bifrost",
                            );

                            return Ok(BifrostClosing {
                                ws_is_closed: true,
                                handle: self.handle,
                            });
                        }
                        // Received unexpected web socket message type
                        Some(Ok(unexpected_message)) => {
                            warn!(
                                message = ?unexpected_message,
                                "received unexpected message type; skipping",
                            );
                            continue;
                        }
                        // Next message was a web socket error
                        Some(Err(err)) => return Err(err.into()),
                        // Web socket stream has closed
                        None => {
                            debug!(
                                "web socket client stream has closed; shutting down bifrost",
                            );

                            return Ok(BifrostClosing {
                                ws_is_closed: true,
                                handle: self.handle,
                            });
                        }
                    }
                }
                // Maybe a response for the web socket client
                maybe_response = self.responses_rx.recv() => {
                    match maybe_response {
                        // Received a response
                        Some(response) => {
                            let payload = serde_json::to_string(&response)
                                .map_err(BifrostError::FriggReadsTaskResponseSerialize)?;
                            let msg = ws::Message::Text(payload);

                            match Self::send_ws_client_message(ws_client, msg).await {
                                // Web socket closed, tear down
                                Some(Ok(_)) => {
                                    debug!(
                                        "before sending response, web socket client has closed; {}",
                                        "shutting down bifrost",
                                    );

                                    return Ok(BifrostClosing {
                                        ws_is_closed: true,
                                        handle: self.handle,
                                    });
                                }
                                // Error sending message to client
                                Some(Err(err)) => return Err(err),
                                // Successfully sent web socket message to client
                                None => {}
                            }
                        }
                        // Channel is empty and closed
                        None => {
                            // Task has terminated prematurely which is an error
                            return Err(BifrostError::FriggReadsTaskRecv);
                        }
                    }
                }
                // NATS message from deployment or workspace subject
                maybe_nats_message_result = self.nats_messages.next() => {
                    match maybe_nats_message_result {
                        // We have a message
                        Some(Ok(nats_message)) => {
                            let ws_message = match self.build_ws_message(nats_message).await {
                                Ok(ws_message) => ws_message,
                                Err(err) => {
                                    warn!(
                                        si.error.message = ?err,
                                        "failed to forward a nats message to web socket; skipping",
                                    );
                                    continue;
                                }
                            };

                            match Self::send_ws_client_message(ws_client, ws_message).await {
                                // Web socket closed, tear down
                                Some(Ok(_)) => {
                                    debug!(
                                        "before sending response, web socket client has closed; {}",
                                        "shutting down bifrost",
                                    );

                                    return Ok(BifrostClosing {
                                        ws_is_closed: true,
                                        handle: self.handle,
                                    });
                                }
                                // Error sending message to client
                                Some(Err(err)) => return Err(err),
                                // Successfully sent web socket message to client
                                None => {}
                            }
                        }
                        // We have a `RecvError`
                        Some(Err(err)) => {
                            warn!(
                                si.error.message = ?err,
                                "encountered a recv error on NATS subscription; skipping",
                            );
                        }
                        // We have a `RecvError`
                        None => {
                            info!("nats listener has closed; bifrost is probably shutting down");
                        }
                    }
                }
                else => break,
            }
        }

        Ok(BifrostClosing {
            ws_is_closed: false,
            handle: self.handle,
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

    async fn send_ws_client_message(
        ws_client: &mut ws::WebSocket,
        ws_message: ws::Message,
    ) -> Option<Result<()>> {
        if let Err(err) = ws_client.send(ws_message).await {
            match err
                .source()
                .and_then(|err| err.downcast_ref::<tungstenite::Error>())
            {
                // If the websocket has cleanly closed, we should cleanly finish as
                // well--this is not an error condition
                Some(tungstenite::Error::ConnectionClosed)
                | Some(tungstenite::Error::AlreadyClosed) => {
                    trace!("websocket has cleanly closed, ending");
                    return Some(Ok(()));
                }
                _ => return Some(Err(BifrostError::WsSendIo(err))),
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct BifrostClosing {
    ws_is_closed: bool,
    handle: BifrostFriggReadsTaskHandle,
}

impl BifrostClosing {
    pub async fn finish(self, ws: WebSocket) -> Result<()> {
        // Cancel and await shutdown of task
        self.handle.await;

        if !self.ws_is_closed {
            ws.close().await.map_err(BifrostError::WsClose)?;
        }

        Ok(())
    }
}

mod task {
    use std::{
        pin::Pin,
        result,
        task::{
            Context,
            Poll,
        },
    };

    use frigg::FriggStore;
    use futures::FutureExt;
    use sdf_core::index::front_end_object_meta;
    use telemetry::prelude::*;
    use thiserror::Error;
    use tokio::{
        sync::mpsc,
        task::JoinHandle,
    };
    use tokio_util::sync::CancellationToken;

    use super::{
        WsFrontEndObjectResponse,
        WsFrontEndOjbectRequest,
    };

    #[derive(Debug, Error)]
    pub(super) enum BifrostFriggReadsTaskError {}

    type Result<T> = result::Result<T, BifrostFriggReadsTaskError>;

    #[derive(Debug)]
    pub(super) struct BifrostFriggReadsTaskHandle {
        join_handle: JoinHandle<()>,
        task_token: CancellationToken,
        internally_cancelled: bool,
    }

    impl BifrostFriggReadsTaskHandle {
        pub(super) fn new(join_handle: JoinHandle<()>, task_token: CancellationToken) -> Self {
            Self {
                join_handle,
                task_token,
                internally_cancelled: false,
            }
        }
    }

    impl Drop for BifrostFriggReadsTaskHandle {
        fn drop(&mut self) {
            self.task_token.cancel();
        }
    }

    impl Future for BifrostFriggReadsTaskHandle {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.internally_cancelled {
                self.task_token.cancel();
                self.internally_cancelled = true;
            }

            match futures::ready!(self.join_handle.poll_unpin(cx)) {
                // Task finished successfully
                Ok(_) => Poll::Ready(()),
                // Task did not finish successfully
                Err(join_err) => {
                    if join_err.is_panic() {
                        warn!("{} panicked reported on join", BifrostFriggReadsTask::NAME);
                    } else if join_err.is_cancelled() {
                        debug!("{} cancelled reported on join", BifrostFriggReadsTask::NAME);
                    } else {
                        warn!(
                            "{} errored for an unknown reason on join handle",
                            BifrostFriggReadsTask::NAME
                        );
                    }

                    Poll::Ready(())
                }
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct BifrostFriggReadsTask {
        frigg: FriggStore,
        requests_rx: mpsc::Receiver<WsFrontEndOjbectRequest>,
        responses_tx: mpsc::Sender<WsFrontEndObjectResponse>,
        token: CancellationToken,
    }

    impl BifrostFriggReadsTask {
        const NAME: &'static str = "sdf_v1_routes::bifrost::proto::bifrost_frigg_reads_task";

        pub(super) fn create(
            frigg: FriggStore,
            requests_rx: mpsc::Receiver<WsFrontEndOjbectRequest>,
            responses_tx: mpsc::Sender<WsFrontEndObjectResponse>,
            token: CancellationToken,
        ) -> Self {
            Self {
                frigg,
                requests_rx,
                responses_tx,
                token,
            }
        }

        pub(super) async fn run(mut self) {
            if let Err(err) = self.try_run().await {
                error!(
                    task = Self::NAME,
                    si.error.message = ?err,
                    "error while running {}",
                    Self::NAME,
                );
            }
        }

        async fn try_run(&mut self) -> Result<()> {
            loop {
                tokio::select! {
                    // Cancellation token has fired, time to shut down
                    _ = self.token.cancelled() => {
                        debug!(task = Self::NAME, "received cancellation");
                        // Close requests channel to ensure to further values cannot be received
                        // and continue to process remaining values until channel is fully drained
                        self.requests_rx.close();
                    }
                    // Maybe next request
                    maybe_request = self.requests_rx.recv() => {
                        match maybe_request {
                            // Next request
                            Some(request) => {
                                if let Err(err) = self.process_request(request).await {
                                    error!(
                                        task = Self::NAME,
                                        si.error.message = ?err,
                                        "error while processing bifrost frigg read request",
                                    );
                                }
                            }
                            // Channel is empty and closed
                            None => {
                                trace!(
                                    task = Self::NAME,
                                    "requests_rx is empty and/or closed; ending task",
                                );
                                break;
                            }
                        }
                    }
                }
            }

            debug!(task = Self::NAME, "shutdown complete");
            Ok(())
        }

        async fn process_request(&self, ws_request: WsFrontEndOjbectRequest) -> Result<()> {
            let ws_response = match front_end_object_meta(
                &self.frigg,
                ws_request.workspace_id,
                ws_request.change_set_id,
                &ws_request.request,
            )
            .await
            {
                Ok(response) => WsFrontEndObjectResponse::Ok {
                    ws_request_id: ws_request.ws_request_id,
                    workspace_id: ws_request.workspace_id,
                    change_set_id: ws_request.change_set_id,
                    response,
                },
                Err(err) => WsFrontEndObjectResponse::Err {
                    ws_request_id: ws_request.ws_request_id,
                    workspace_id: ws_request.workspace_id,
                    change_set_id: ws_request.change_set_id,
                    error: err.to_string(),
                    request: ws_request.request,
                },
            };

            if let Err(err) = self.responses_tx.send(ws_response).await {
                error!(
                    task = Self::NAME,
                    si.error.message = ?err,
                    "error sending bifrost frigg read response; cancelling task",
                );
                self.token.cancel();
            };

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use si_frontend_mv_types::object::FrontendObject;

    use super::*;

    // The following tests are here to help to print out what the request/response messages should
    // look like.
    //
    // To see, uncommented the `panic!()` lines of whichever tests and run the unit tests. They
    // will fail and print out the JSON representation using a pretty output format.
    mod serialize {
        use super::*;

        fn ws_request() -> WsFrontEndOjbectRequest {
            WsFrontEndOjbectRequest {
                ws_request_id: Some("123".to_string()),
                workspace_id: "01JWW640R16P28HXPTZV1EAVDX".parse().unwrap(),
                change_set_id: "01JWW6522C1XEG62RC01JMGBTV".parse().unwrap(),
                request: FrontEndObjectRequest {
                    kind: "DooferDoodle".to_string(),
                    id: "1111".to_string(),
                    checksum: Some("1111_chk".to_string()),
                },
            }
        }

        #[test]
        fn ws_front_end_object_request() {
            let serialized =
                serde_json::to_string_pretty(&ws_request()).expect("failed to serialize");

            println!("{serialized}");

            // panic!("let's see the serialized!");
        }

        #[test]
        fn ws_front_end_object_response_ok() {
            let response = WsFrontEndObjectResponse::Ok {
                ws_request_id: Some("123".to_string()),
                workspace_id: "01JWW640R16P28HXPTZV1EAVDX".parse().unwrap(),
                change_set_id: "01JWW6522C1XEG62RC01JMGBTV".parse().unwrap(),
                response: FrontEndObjectMeta {
                    workspace_snapshot_address: "wk_snap_addr".to_string(),
                    index_checksum: "idx_chk".to_string(),
                    front_end_object: FrontendObject {
                        kind: "DooferDoodle".to_string(),
                        id: "1111".to_string(),
                        checksum: "1111_chk".to_string(),
                        data: json!({
                            "one": "two",
                        }),
                    },
                },
            };

            let serialized = serde_json::to_string_pretty(&response).expect("failed to serialize");

            println!("{serialized}");

            // panic!("let's see the serialized!");
        }

        #[test]
        fn ws_front_end_object_response_err() {
            let response = WsFrontEndObjectResponse::Err {
                ws_request_id: Some("123".to_string()),
                workspace_id: "01JWW640R16P28HXPTZV1EAVDX".parse().unwrap(),
                change_set_id: "01JWW6522C1XEG62RC01JMGBTV".parse().unwrap(),
                error: "you made a boo boo".to_string(),
                request: ws_request().request,
            };

            let serialized = serde_json::to_string_pretty(&response).expect("failed to serialize");

            println!("{serialized}");

            // panic!("let's see the serialized!");
        }
    }
}
