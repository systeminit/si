use super::WsError;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use dal::{WorkspacePk, WsEventError};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use std::{collections::hash_map::Entry, collections::HashMap, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::Mutex;
use y::{WarpSink, WarpStream};
use y_sync::net::BroadcastGroup;

use crate::server::{
    extract::{Nats, WsAuthorization},
    state::ShutdownBroadcast,
};

pub mod y;

// TODO / Note(paulo: this is a hacky implementation, where the front-end is the source of truth and this
// service only broadcasts stuff. Ideally we should be able to get the doc from the dal, and save
// to it with a debounce, becoming the source of truth

// TODO(paulo): scale this horizontally, right now it will keep a different buffer in each sdf if there
// are multiple, the solution is reimplementing yrs-warp (crdt/y.rs) using nats as an intermediary (instead of
// piping yjs events from ws, pipes from the nats subscription, or the opposite direction)

// TODO: possibly use webrtc signaling service for y-webrtc: https://github.com/y-crdt/yrs-warp/blob/827bea8bf5255b4e6451d94287d05a9cd2f60a8c/src/signaling.rs

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("failed to subscribe to subject {1}")]
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

#[instrument(skip(wsu, _nats, broadcast_groups))]
pub async fn crdt(
    wsu: WebSocketUpgrade,
    Nats(_nats): Nats,
    WsAuthorization(claim): WsAuthorization,
    Query(Id { id }): Query<Id>,
    State(shutdown_broadcast): State<ShutdownBroadcast>,
    State(broadcast_groups): State<BroadcastGroups>,
) -> Result<impl IntoResponse, WsError> {
    Ok(crdt_inner(
        wsu,
        claim.workspace_pk,
        id,
        shutdown_broadcast,
        broadcast_groups,
    )
    .await?)
}

#[allow(clippy::unused_async)]
pub async fn crdt_inner(
    wsu: WebSocketUpgrade,
    workspace_pk: WorkspacePk,
    id: String,
    shutdown_broadcast: ShutdownBroadcast,
    broadcast_groups: BroadcastGroups,
) -> Result<impl IntoResponse, CrdtError> {
    let _shutdown = shutdown_broadcast.subscribe();
    Ok(wsu.on_upgrade(move |socket| async move {
        let (sink, stream) = socket.split();
        let sink = Arc::new(Mutex::new(WarpSink::from(sink)));
        let stream = WarpStream::from(stream);

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
        match sub.completed().await {
            Ok(_) => trace!("broadcasting for channel finished successfully"),
            Err(e) => debug!("broadcasting for channel finished abruptly: {}", e),
        }
    }))
}

/// Adapted from: https://github.com/y-crdt/yrs-warp/blob/14a1abdf9085d71b6071e27c3e53ac5d0e07735d/src/ws.rs
#[cfg(test)]
mod test {
    use crate::server::service::{ws::crdt::BroadcastGroups, ws::WsError};
    use crate::server::state::ShutdownBroadcast;
    use axum::{
        extract::FromRef, extract::State, extract::WebSocketUpgrade, response::IntoResponse,
        routing::get, Router,
    };
    use dal::WorkspacePk;
    use futures::{ready, SinkExt, Stream, StreamExt};
    use futures::{stream::SplitSink, stream::SplitStream, Sink};
    use std::{
        collections::HashMap, net::SocketAddr, pin::Pin, str::FromStr, sync::Arc, task::Context,
        task::Poll, time::Duration,
    };
    use tokio::{
        net::TcpStream, sync::broadcast, sync::Mutex, sync::Notify, sync::RwLock, task,
        task::JoinHandle, time::sleep, time::timeout,
    };
    use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
    use y_sync::{awareness::Awareness, net::BroadcastGroup, net::Connection, sync::Error};
    use yrs::{updates::encoder::Encode, Doc, GetString, Text, Transact, UpdateSubscription};

    pub async fn crdt(
        wsu: WebSocketUpgrade,
        State(shutdown_broadcast): State<ShutdownBroadcast>,
        State(broadcast_groups): State<BroadcastGroups>,
    ) -> Result<impl IntoResponse, WsError> {
        Ok(super::crdt_inner(
            wsu,
            WorkspacePk::NONE,
            "my-room".to_owned(),
            shutdown_broadcast,
            broadcast_groups,
        )
        .await?)
    }

    #[derive(Clone, FromRef)]
    pub struct AppState {
        broadcast_groups: BroadcastGroups,
        shutdown_broadcast: ShutdownBroadcast,
    }

    async fn start_server(
        addr: &str,
        bcast: Arc<BroadcastGroup>,
    ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let addr = SocketAddr::from_str(addr)?;

        let mut map = HashMap::new();
        map.insert(format!("{}-my-room", WorkspacePk::NONE), bcast);

        let (shutdown_broadcast_tx, _shutdown_broadcast_rx) = broadcast::channel(1);
        let state = AppState {
            broadcast_groups: Arc::new(Mutex::new(map)),
            shutdown_broadcast: ShutdownBroadcast::new(shutdown_broadcast_tx),
        };

        let routes = Router::new().route("/my-room", get(crdt)).with_state(state);

        Ok(tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(routes.into_make_service())
                .await
                .expect("unable to start axum test server");
        }))
    }

    struct TungsteniteSink(SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>);

    impl Sink<Vec<u8>> for TungsteniteSink {
        type Error = Error;

        fn poll_ready(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            let sink = unsafe { Pin::new_unchecked(&mut self.0) };
            let result = ready!(sink.poll_ready(cx));
            match result {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
            }
        }

        fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
            let sink = unsafe { Pin::new_unchecked(&mut self.0) };
            let result = sink.start_send(Message::binary(item));
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Other(Box::new(e))),
            }
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            let sink = unsafe { Pin::new_unchecked(&mut self.0) };
            let result = ready!(sink.poll_flush(cx));
            match result {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
            }
        }

        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            let sink = unsafe { Pin::new_unchecked(&mut self.0) };
            let result = ready!(sink.poll_close(cx));
            match result {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
            }
        }
    }

    struct TungsteniteStream(SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>);
    impl Stream for TungsteniteStream {
        type Item = Result<Vec<u8>, Error>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let stream = unsafe { Pin::new_unchecked(&mut self.0) };
            let result = ready!(stream.poll_next(cx));
            match result {
                None => Poll::Ready(None),
                Some(Ok(msg)) => Poll::Ready(Some(Ok(msg.into_data()))),
                Some(Err(e)) => Poll::Ready(Some(Err(Error::Other(Box::new(e))))),
            }
        }
    }

    async fn client(
        addr: &str,
        doc: Doc,
    ) -> Result<Connection<TungsteniteSink, TungsteniteStream>, Box<dyn std::error::Error>> {
        let (stream, _) = tokio_tungstenite::connect_async(addr).await?;
        let (sink, stream) = stream.split();
        let sink = TungsteniteSink(sink);
        let stream = TungsteniteStream(stream);
        Ok(Connection::new(
            Arc::new(RwLock::new(Awareness::new(doc))),
            sink,
            stream,
        ))
    }

    fn create_notifier(doc: &Doc) -> (Arc<Notify>, UpdateSubscription) {
        let n = Arc::new(Notify::new());
        let sub = {
            let n = n.clone();
            doc.observe_update_v1(move |_, _| n.notify_waiters())
                .expect("unable to observe update v1")
        };
        (n, sub)
    }

    const TIMEOUT: Duration = Duration::from_secs(5);

    #[tokio::test]
    async fn change_introduced_by_server_reaches_subscribed_clients(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let doc = Doc::with_client_id(1);
        let text = doc.get_or_insert_text("test");
        let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
        let bcast = BroadcastGroup::new(awareness.clone(), 10).await;
        let _server = start_server("0.0.0.0:6600", Arc::new(bcast)).await?;

        let doc = Doc::new();
        let (n, _sub) = create_notifier(&doc);
        let c1 = client("ws://localhost:6600/my-room", doc).await?;

        {
            let lock = awareness.write().await;
            text.push(&mut lock.doc().transact_mut(), "abc");
        }

        timeout(TIMEOUT, n.notified()).await?;

        {
            let awareness = c1.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "abc".to_string());
        }

        Ok(())
    }

    #[tokio::test]
    async fn subscribed_client_fetches_initial_state() -> Result<(), Box<dyn std::error::Error>> {
        let doc = Doc::with_client_id(1);
        let text = doc.get_or_insert_text("test");

        text.push(&mut doc.transact_mut(), "abc");

        let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
        let bcast = BroadcastGroup::new(awareness.clone(), 10).await;
        let _server = start_server("0.0.0.0:6601", Arc::new(bcast)).await?;

        let doc = Doc::new();
        let (n, _sub) = create_notifier(&doc);
        let c1 = client("ws://localhost:6601/my-room", doc).await?;

        timeout(TIMEOUT, n.notified()).await?;

        {
            let awareness = c1.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "abc".to_string());
        }

        Ok(())
    }

    #[tokio::test]
    async fn changes_from_one_client_reach_others() -> Result<(), Box<dyn std::error::Error>> {
        let doc = Doc::with_client_id(1);
        let _text = doc.get_or_insert_text("test");

        let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
        let bcast = BroadcastGroup::new(awareness.clone(), 10).await;
        let _server = start_server("0.0.0.0:6602", Arc::new(bcast)).await?;

        let d1 = Doc::with_client_id(2);
        let c1 = client("ws://localhost:6602/my-room", d1).await?;
        // by default changes made by document on the client side are not propagated automatically
        let _sub11 = {
            let sink = c1.sink();
            let a = c1.awareness().write().await;
            let doc = a.doc();
            doc.observe_update_v1(move |_txn, e| {
                let update = e.update.to_owned();
                if let Some(sink) = sink.upgrade() {
                    task::spawn(async move {
                        let msg =
                            y_sync::sync::Message::Sync(y_sync::sync::SyncMessage::Update(update))
                                .encode_v1();
                        let mut sink = sink.lock().await;
                        sink.send(msg).await.expect("unable to send msg to sink");
                    });
                }
            })
            .expect("unable to observe update v1")
        };

        let d2 = Doc::with_client_id(3);
        let (n2, _sub2) = create_notifier(&d2);
        let c2 = client("ws://localhost:6602/my-room", d2).await?;

        {
            let a = c1.awareness().write().await;
            let doc = a.doc();
            let text = doc.get_or_insert_text("test");
            text.push(&mut doc.transact_mut(), "def");
        }

        timeout(TIMEOUT, n2.notified()).await?;

        {
            let awareness = c2.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "def".to_string());
        }

        Ok(())
    }

    #[tokio::test]
    async fn client_failure_doesnt_affect_others() -> Result<(), Box<dyn std::error::Error>> {
        let doc = Doc::with_client_id(1);
        let _text = doc.get_or_insert_text("test");

        let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
        let bcast = BroadcastGroup::new(awareness.clone(), 10).await;
        let _server = start_server("0.0.0.0:6603", Arc::new(bcast)).await?;

        let d1 = Doc::with_client_id(2);
        let c1 = client("ws://localhost:6603/my-room", d1).await?;
        // by default changes made by document on the client side are not propagated automatically
        let _sub11 = {
            let sink = c1.sink();
            let a = c1.awareness().write().await;
            let doc = a.doc();
            doc.observe_update_v1(move |_txn, e| {
                let update = e.update.to_owned();
                if let Some(sink) = sink.upgrade() {
                    task::spawn(async move {
                        let msg =
                            y_sync::sync::Message::Sync(y_sync::sync::SyncMessage::Update(update))
                                .encode_v1();
                        let mut sink = sink.lock().await;
                        sink.send(msg).await.expect("unable to send msg to sink");
                    });
                }
            })
            .expect("unable to observe update v1")
        };

        let d2 = Doc::with_client_id(3);
        let (n2, sub2) = create_notifier(&d2);
        let c2 = client("ws://localhost:6603/my-room", d2).await?;

        let d3 = Doc::with_client_id(4);
        let (n3, sub3) = create_notifier(&d3);
        let c3 = client("ws://localhost:6603/my-room", d3).await?;

        {
            let a = c1.awareness().write().await;
            let doc = a.doc();
            let text = doc.get_or_insert_text("test");
            text.push(&mut doc.transact_mut(), "abc");
        }

        // on the first try both C2 and C3 should receive the update
        //timeout(TIMEOUT, n2.notified()).await.unwrap();
        //timeout(TIMEOUT, n3.notified()).await.unwrap();
        sleep(TIMEOUT).await;

        {
            let awareness = c2.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "abc".to_string());
        }
        {
            let awareness = c3.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "abc".to_string());
        }

        // drop client, causing abrupt ending
        drop(c3);
        drop(n3);
        drop(sub3);
        // C2 notification subscription has been realized, we need to refresh it
        drop(n2);
        drop(sub2);

        let (n2, _sub2) = {
            let a = c2.awareness().write().await;
            let doc = a.doc();
            create_notifier(doc)
        };

        {
            let a = c1.awareness().write().await;
            let doc = a.doc();
            let text = doc.get_or_insert_text("test");
            text.push(&mut doc.transact_mut(), "def");
        }

        timeout(TIMEOUT, n2.notified()).await.expect("timeout");

        {
            let awareness = c2.awareness().read().await;
            let doc = awareness.doc();
            let text = doc.get_or_insert_text("test");
            let str = text.get_string(&doc.transact());
            assert_eq!(str, "abcdef".to_string());
        }

        Ok(())
    }
}
