use super::WsError;
use axum::{
    extract::{ws::Message, Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use dal::{WorkspacePk, WsEventError};
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use si_data_nats::{NatsClient, NatsError, Subscriber};
use std::{collections::hash_map::Entry, collections::HashMap, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::broadcast, sync::Mutex, task::JoinSet};
use y::{YSink, YStream};
use y_sync::net::BroadcastGroup;

use crate::server::{
    extract::{Nats, WsAuthorization},
    state::ShutdownBroadcast,
};

pub mod y;

// TODO: move source of truth to server, generating BroadcastGroup with data from the dal and
// automatically update database if our websocket connection changes something instead of using
// front-end

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("broadcast error: {0}")]
    Broadcast(#[from] broadcast::error::SendError<Message>),
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

pub type BroadcastGroups = Arc<Mutex<HashMap<String, Arc<BroadcastGroup>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    id: String,
}

#[instrument(skip(wsu, nats, broadcast_groups))]
pub async fn crdt(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    WsAuthorization(claim): WsAuthorization,
    Query(Id { id }): Query<Id>,
    State(shutdown_broadcast): State<ShutdownBroadcast>,
    State(broadcast_groups): State<BroadcastGroups>,
) -> Result<impl IntoResponse, WsError> {
    let workspace_pk = claim.workspace_pk;
    let channel_name = format!("crdt-{workspace_pk}-{id}");
    let subscription = nats.subscribe(&channel_name).await?;
    let ws_subscription = nats.subscribe(&channel_name).await?;
    let shutdown = shutdown_broadcast.subscribe();

    Ok(wsu.on_upgrade(move |socket| async move {
        let (sink, stream) = socket.split();
        crdt_handle(
            sink,
            stream,
            nats,
            broadcast_groups,
            channel_name,
            subscription,
            ws_subscription,
            workspace_pk,
            id,
            shutdown,
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
    channel_name: String,
    subscription: Subscriber,
    mut ws_subscription: Subscriber,
    workspace_pk: WorkspacePk,
    id: String,
    mut shutdown: broadcast::Receiver<()>,
) where
    W: Sink<Message> + Unpin + Send + 'static,
    R: Stream<Item = Result<Message, axum::Error>> + Unpin + Send + 'static,
    CrdtError: From<<W as Sink<Message>>::Error>,
{
    let mut tasks = JoinSet::new();

    tasks.spawn(async move {
        while let Some(message) = ws_subscription.next().await {
            sink.send(Message::Binary(message.payload().to_owned()))
                .await?;
        }

        let result: CrdtResult<()> = Ok(());
        result
    });

    let (ws_nats, ws_channel_name) = (nats.clone(), channel_name.clone());
    tasks.spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Message::Binary(vec) = msg? {
                ws_nats.publish(&ws_channel_name, vec).await?;
            }
        }

        Ok(())
    });

    tasks.spawn(async move { Ok(shutdown.recv().await?) });

    let sink = Arc::new(Mutex::new(YSink::new(nats, channel_name)));
    let stream = YStream::new(subscription);

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
        result = sub.completed() => {
            match result {
                Ok(_) => info!("broadcasting for channel finished successfully"),
                Err(e) => error!("broadcasting for channel finished abruptly: {}", e),
            }
        }
        Some(result) = tasks.join_next() => {
            match result {
                Ok(Err(err)) => {
                    error!("Task failed: {err}");
                }
                Err(err) => {
                    error!("Unable to join task: {err}");
                }
                Ok(Ok(())) => {},
            }
        }
        else => {},
    }

    tasks.shutdown().await;
}
