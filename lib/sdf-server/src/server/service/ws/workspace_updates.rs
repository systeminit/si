use super::WsError;
use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
};
use dal::WorkspacePk;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::sync::broadcast;

use crate::server::{
    extract::{Nats, WsAuthorization},
    state::ShutdownBroadcast,
};

#[instrument(skip(wsu, nats))]
#[allow(clippy::unused_async)]
pub async fn workspace_updates(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    WsAuthorization(claim): WsAuthorization,
    State(shutdown_broadcast): State<ShutdownBroadcast>,
) -> Result<impl IntoResponse, WsError> {
    async fn handle_socket(
        socket: WebSocket,
        nats: NatsClient,
        mut shutdown: broadcast::Receiver<()>,
        workspace_pk: WorkspacePk,
    ) {
        tokio::select! {
            _ = run_workspace_updates_proto(socket, nats, workspace_pk) => {
                trace!("finished workspace_updates proto");
            }
            _ = shutdown.recv() => {
                trace!("workspace_updates received shutdown, ending session");
            }
            else => {
                trace!("returning from workspace_updates, all select arms closed");
            }
        }
    }

    let shutdown = shutdown_broadcast.subscribe();
    Ok(wsu.on_upgrade(move |socket| handle_socket(socket, nats, shutdown, claim.workspace_pk)))
}

async fn run_workspace_updates_proto(
    mut socket: WebSocket,
    nats: NatsClient,
    workspace_pk: WorkspacePk,
) {
    let proto = match workspace_updates::run(nats, workspace_pk).start().await {
        Ok(started) => started,
        Err(err) => {
            // This is likely due to nats failing to subscribe to the required topic, which is
            // suspicious
            warn!(error = ?err, "protocol failed to start");
            return;
        }
    };
    let proto = match proto.process(&mut socket).await {
        Ok(processed) => processed,
        Err(err) => {
            // An error is most likely returned when the client side terminates the websocket
            // session or if a network partition occurs, so this is our "normal" behavior
            trace!(error = ?err, "failed to cleanly complete update stream");
            return;
        }
    };
    if let Err(err) = proto.finish(socket).await {
        // We'd like finish to complete cleanly
        warn!(error = ?err, "failed to finish protocol");
    }
}

mod workspace_updates {
    use std::error::Error;

    use axum::extract::ws::{self, WebSocket};
    use dal::{ChangeSetPk, UserPk, WorkspacePk, WsEvent, WsEventError};
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};
    use si_data_nats::{NatsClient, NatsError, Subscriber};
    use telemetry::prelude::*;
    use thiserror::Error;
    use tokio_tungstenite::tungstenite;

    #[remain::sorted]
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "kind", content = "data")]
    pub enum WebsocketEventRequest {
        #[serde(rename_all = "camelCase")]
        Cursor {
            user_pk: UserPk,
            user_name: String,
            change_set_pk: Option<ChangeSetPk>,
            container: Option<String>,
            x: String,
            y: String,
        },
    }

    pub fn run(nats: NatsClient, workspace_pk: WorkspacePk) -> WorkspaceUpdates {
        WorkspaceUpdates { nats, workspace_pk }
    }

    #[remain::sorted]
    #[derive(Debug, Error)]
    pub enum WorkspaceUpdatesError {
        #[error("axum error: {0}")]
        Axum(#[from] axum::Error),
        #[error("nats error: {0}")]
        Nats(#[from] si_data_nats::Error),
        #[error("serde json error: {0}")]
        Serde(#[from] serde_json::Error),
        #[error("failed to subscribe to subject {1}")]
        Subscribe(#[source] NatsError, String),
        #[error("error when closing websocket")]
        WsClose(#[source] axum::Error),
        #[error("wsevent error: {0}")]
        WsEvent(#[from] WsEventError),
        #[error("error when sending websocket message")]
        WsSendIo(#[source] axum::Error),
    }

    type Result<T> = std::result::Result<T, WorkspaceUpdatesError>;

    #[derive(Debug)]
    pub struct WorkspaceUpdates {
        nats: NatsClient,
        workspace_pk: WorkspacePk,
    }

    impl WorkspaceUpdates {
        pub async fn start(self) -> Result<WorkspaceUpdatesStarted> {
            let subject = format!("si.workspace_pk.{}.>", self.workspace_pk);
            let subscriber = self
                .nats
                .subscribe(&subject)
                .await
                .map_err(|err| WorkspaceUpdatesError::Subscribe(err, subject))?;

            Ok(WorkspaceUpdatesStarted {
                nats: self.nats.clone(),
                workspace_pk: self.workspace_pk,
                subscriber,
            })
        }
    }

    #[derive(Debug)]
    pub struct WorkspaceUpdatesStarted {
        workspace_pk: WorkspacePk,
        nats: NatsClient,
        subscriber: Subscriber,
    }

    impl WorkspaceUpdatesStarted {
        pub async fn process(mut self, ws: &mut WebSocket) -> Result<WorkspaceUpdatesClosing> {
            // Send all messages down the WebSocket until and unless an error is encountered, the
            // client websocket connection is closed, or the nats subscriber naturally closes
            loop {
                tokio::select! {
                    msg = ws.recv() => {
                        match msg {
                            Some(Ok(message)) => if let ws::Message::Text(msg) = message {
                                let event: WebsocketEventRequest = match serde_json::from_str(&msg) {
                                    Ok(event) => event,
                                    Err(err) => {
                                        error!("Unable to deserialize websocket message: {err}");
                                        continue;
                                    }
                                };
                                match event {
                                    WebsocketEventRequest::Cursor { user_pk, user_name, change_set_pk, container, x, y } => {
                                        let subject = format!("si.workspace_pk.{}.event", self.workspace_pk);
                                        let event = WsEvent::cursor(self.workspace_pk, change_set_pk.unwrap_or(ChangeSetPk::NONE), user_pk, user_name, x, y, container).await?;
                                        self.nats.publish(subject, serde_json::to_vec(&event)?).await?;
                                    }
                                }
                            },
                            Some(Err(err)) => return Err(err.into()),
                            None => return Ok(WorkspaceUpdatesClosing { ws_is_closed: true }),
                        }
                    }
                    Some(nats_msg) = self.subscriber.next() => {
                        let msg = ws::Message::Text(String::from_utf8_lossy(nats_msg.payload()).to_string());

                        if let Err(err) = ws.send(msg).await {
                            match err
                                .source()
                                .and_then(|err| err.downcast_ref::<tungstenite::Error>())
                            {
                                Some(ws_err) => match ws_err {
                                    // If the websocket has cleanly closed, we should cleanly finish as
                                    // well--this is not an error condition
                                    tungstenite::Error::ConnectionClosed
                                    | tungstenite::Error::AlreadyClosed => {
                                        trace!("websocket has cleanly closed, ending");
                                        return Ok(WorkspaceUpdatesClosing { ws_is_closed: true });
                                    }
                                    _ => return Err(WorkspaceUpdatesError::WsSendIo(err)),
                                },
                                None => return Err(WorkspaceUpdatesError::WsSendIo(err)),
                            }
                        }
                    }
                    else => break,
                }
            }

            Ok(WorkspaceUpdatesClosing {
                ws_is_closed: false,
            })
        }
    }

    #[derive(Debug)]
    pub struct WorkspaceUpdatesClosing {
        ws_is_closed: bool,
    }

    impl WorkspaceUpdatesClosing {
        pub async fn finish(self, ws: WebSocket) -> Result<()> {
            if !self.ws_is_closed {
                ws.close().await.map_err(WorkspaceUpdatesError::WsClose)?;
            }
            Ok(())
        }
    }
}
