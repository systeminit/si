use std::{
    collections::hash_map::Entry,
    sync::Arc,
};

use axum::{
    extract::{
        Query,
        State,
        WebSocketUpgrade,
        ws::{
            self,
            Message,
        },
    },
    response::IntoResponse,
};
use dal::{
    WorkspacePk,
    WsEventError,
};
use futures::{
    Sink,
    SinkExt,
    Stream,
    StreamExt,
};
use nats_multiplexer_client::MultiplexerRequestPayload;
use sdf_core::{
    BroadcastGroups,
    nats_multiplexer::NatsMultiplexerClients,
};
use sdf_extract::{
    request::TokenFromQueryParam,
    services::Nats,
    workspace::{
        TargetWorkspaceIdFromToken,
        WorkspaceAuthorization,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::{
    NatsClient,
    NatsError,
    Subject,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    Mutex,
    broadcast,
};
use tokio_stream::wrappers::{
    BroadcastStream,
    errors::BroadcastStreamRecvError,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use y::{
    YSink,
    YStream,
};
use y_sync::net::BroadcastGroup;

use crate::WsError;

pub mod y;

// TODO: move source of truth to server, generating BroadcastGroup with data from the dal and
// automatically update database if our websocket connection changes something instead of using
// front-end

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("broadcast send error: {0}")]
    BroadcastSend(#[from] broadcast::error::SendError<Message>),
    #[error("broadcast stream recv error: {0}")]
    BrodcastStreamRecv(#[from] BroadcastStreamRecvError),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("Shutdown recv error: {0}")]
    Recv(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("failed to subscribe to subject: {0} {1}")]
    Subscribe(#[source] NatsError, String),
    #[error("wsevent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type CrdtResult<T, E = CrdtError> = Result<T, E>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    id: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn crdt(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    _: TokenFromQueryParam, // This tells it to pull the token from the "token" param
    _: TargetWorkspaceIdFromToken,
    auth: WorkspaceAuthorization,
    Query(Id { id }): Query<Id>,
    State(shutdown_token): State<CancellationToken>,
    State(broadcast_groups): State<BroadcastGroups>,
    State(nats_multiplexer_clients): State<NatsMultiplexerClients>,
) -> Result<impl IntoResponse, WsError> {
    let workspace_pk = auth.workspace_id;
    let channel_name = Subject::from(format!("crdt.{workspace_pk}.{id}"));

    let receiver = nats_multiplexer_clients
        .crdt
        .try_lock()?
        .receiver(channel_name.clone())
        .await?;
    let ws_receiver = receiver.resubscribe();

    Ok(wsu.on_upgrade(move |socket| async move {
        let (sink, stream) = socket.split();
        crdt_handle(
            sink,
            stream,
            nats,
            broadcast_groups,
            channel_name,
            receiver,
            ws_receiver,
            workspace_pk,
            id,
            shutdown_token,
        )
        .await
    }))
}

#[allow(clippy::too_many_arguments)]
pub async fn crdt_handle<W, R>(
    mut sink: W,
    mut stream: R,
    nats: NatsClient,
    broadcast_groups: BroadcastGroups,
    subject: Subject,
    receiver: broadcast::Receiver<MultiplexerRequestPayload>,
    ws_receiver: broadcast::Receiver<MultiplexerRequestPayload>,
    workspace_pk: WorkspacePk,
    id: String,
    token: CancellationToken,
) where
    W: Sink<Message> + Unpin + Send + 'static,
    R: Stream<Item = Result<Message, axum::Error>> + Unpin + Send + 'static,
    CrdtError: From<<W as Sink<Message>>::Error>,
{
    let tracker = TaskTracker::new();

    let mut ws_receiver_stream = BroadcastStream::new(ws_receiver);

    // Spawn "writes-to-client" task which consumes from nats
    let to_client_token = token.clone();
    tracker.spawn(async move {
        loop {
            tokio::select! {
                _ = to_client_token.cancelled() => {
                    trace!("web socket writes-to-client has received cancellation");
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
                    if let Err(_item) = sink.send(ws::Message::Close(Some(close_frame))).await {
                        // Not much we can or want to do here--we are in the process of
                        // shutting down
                        warn!(
                            "error while closing websocket connection during graceful shutdown",
                        );
                    }

                    break;
                }
                maybe_message_result = ws_receiver_stream.next() => {
                    match maybe_message_result {
                        Some(Ok(payload)) => {
                            let bytes = payload.nats_message.into_inner().payload.into();
                            if let Err(_item) = sink.send(Message::Binary(bytes)).await {
                                warn!("failed to send message from nats to client");
                            }
                        }
                        Some(Err(err)) => {
                            warn!(error = ?err, "error while processing message from nats");
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Spawn "reads-from-client" task which publishes to nats
    let from_client_token = token.clone();
    let from_client_nats = nats.clone();
    let from_client_subject = subject.clone();
    tracker.spawn(async move {
        loop {
            tokio::select! {
                _ = from_client_token.cancelled() => {
                    trace!("web socket reads-from-client has received cancellation");
                    break;
                }
                maybe_message_result = stream.next() => {
                    match maybe_message_result {
                        Some(Ok(msg)) => {
                            if let Message::Binary(vec) = msg {
                                if let Err(err) = from_client_nats
                                    .publish(from_client_subject.clone(), vec.into())
                                    .await
                                {
                                    warn!(
                                        error = ?err,
                                        "error publishing message from client to nats",
                                    );
                                }
                            }
                        }
                        Some(Err(err)) => {
                            warn!(error = ?err, "error while processing message from client");
                        }
                        None => break,
                    }
                }
            }
        }
    });

    tracker.close();

    let sink = Arc::new(Mutex::new(YSink::new(nats, subject)));
    let stream = YStream::new(receiver);

    let bcast: Arc<BroadcastGroup> = match broadcast_groups
        .lock()
        .await
        .entry(format!("{workspace_pk}-{id}"))
    {
        Entry::Occupied(e) => e.get().clone(),
        Entry::Vacant(e) => e
            .insert(Arc::new(BroadcastGroup::new(Default::default(), 32).await))
            .clone(),
    };

    let sub = bcast.subscribe(sink, stream);
    tokio::select! {
        _ = token.cancelled() => {
            trace!("web socket has received cancellation");
        }
        result = sub.completed() => {
            match result {
                Ok(_) => info!("broadcasting for channel finished successfully"),
                Err(e) => error!("broadcasting for channel finished abruptly: {}", e),
            }
        }
    }

    tracker.wait().await;
}
