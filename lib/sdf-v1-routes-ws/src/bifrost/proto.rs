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
    WorkspacePk,
};
use frigg::FriggStore;
use nats_multiplexer_client::{
    MultiplexerClient,
    MultiplexerClientError,
};
use sdf_core::index::FrontEndObjectMeta;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::Subject;
use si_frontend_mv_types::object::patch::DATA_CACHE_SUBJECT_PREFIX;
use si_frontend_types::FrontEndObjectRequest;
use task::{
    BifrostFriggReadsTask,
    BifrostFriggReadsTaskHandle,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    Mutex,
    broadcast,
    mpsc,
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
    #[error("Token cancellation error: {0}")]
    TokenCancellation(#[from] tokio::task::JoinError),
    #[error("TryLock error: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("Error closing websocket: {0}")]
    WsClose(#[source] axum::Error),
    #[error("Error sending websocket message: {0}")]
    WsSendIo(#[source] axum::Error),
}

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

pub fn run(frigg: FriggStore, workspace_id: WorkspacePk, token: CancellationToken) -> Bifrost {
    Bifrost {
        frigg,
        workspace_id,
        token,
    }
}

#[derive(Debug)]
pub struct Bifrost {
    frigg: FriggStore,
    workspace_id: WorkspacePk,
    token: CancellationToken,
}

impl Bifrost {
    pub async fn start(
        self,
        bifrost_multiplexer_client: Arc<Mutex<MultiplexerClient>>,
    ) -> Result<BifrostStarted> {
        // Subject is wildcarded, but as of this moment could be either patch_message or
        // index_update for now, just pass everything right along
        let subject = Subject::from(format!(
            "{}.workspace_id.{}.*",
            DATA_CACHE_SUBJECT_PREFIX, self.workspace_id
        ));
        let nats_sub = bifrost_multiplexer_client
            .try_lock()?
            .receiver(subject)
            .await?;

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
            nats_sub,
            requests_tx,
            responses_rx,
            handle,
            token: self.token,
        })
    }
}

#[derive(Debug)]
pub struct BifrostStarted {
    nats_sub: broadcast::Receiver<si_data_nats::Message>,
    requests_tx: mpsc::Sender<WsFrontEndOjbectRequest>,
    responses_rx: mpsc::Receiver<WsFrontEndObjectResponse>,
    handle: BifrostFriggReadsTaskHandle,
    token: CancellationToken,
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
                maybe_client_message = ws_client.recv() => {
                    match maybe_client_message {
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

                            match Self::send_client_message(ws_client, msg).await {
                                // Web socket closed, tear down
                                Some(Ok(_)) => {
                                    return Ok(BifrostClosing {
                                        ws_is_closed: true,
                                        handle: self.handle,
                                    });
                                }
                                // Error sending message to client
                                Some(Err(err)) => return Err(err),
                                // Sucessfully sent web socket message to client
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
                // NATS message from subscription
                recv_result = self.nats_sub.recv() => {
                    let nats_msg =  recv_result?;
                    let msg = ws::Message::Text(
                        String::from_utf8_lossy(nats_msg.payload()).to_string()
                    );

                    match Self::send_client_message(ws_client, msg).await {
                        // Web socket closed, tear down
                        Some(Ok(_)) => {
                            return Ok(BifrostClosing {
                                ws_is_closed: true,
                                handle: self.handle,
                            });
                        }
                        // Error sending message to client
                        Some(Err(err)) => return Err(err),
                        // Sucessfully sent web socket message to client
                        None => {}
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

    async fn send_client_message(
        ws_client: &mut ws::WebSocket,
        msg: ws::Message,
    ) -> Option<Result<()>> {
        if let Err(err) = ws_client.send(msg).await {
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
                    _ = self.token.cancelled() => {}
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
                                debug!(
                                    task = Self::NAME,
                                    "requests_rx is empty and/or closed; cancelling task",
                                );
                                self.token.cancel();
                            }
                        }
                    }
                }
            }
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
