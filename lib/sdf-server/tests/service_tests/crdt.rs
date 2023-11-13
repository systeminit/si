use axum::{
    extract::FromRef, extract::State, extract::WebSocketUpgrade, response::IntoResponse,
    routing::get, Router,
};
use dal::WorkspacePk;
use futures::{ready, SinkExt, Stream, StreamExt};
use futures::{stream::SplitSink, stream::SplitStream, Sink};
/// Adapted from: https://github.com/y-crdt/yrs-warp/blob/14a1abdf9085d71b6071e27c3e53ac5d0e07735d/src/ws.rs
use sdf_server::server::service::{ws::crdt::crdt_inner, ws::crdt::BroadcastGroups, ws::WsError};
use sdf_server::server::state::ShutdownBroadcast;
use si_data_nats::{NatsClient, NatsConfig};
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
    State(nats): State<NatsClient>,
    State(workspace_pk): State<WorkspacePk>,
) -> Result<impl IntoResponse, WsError> {
    Ok(crdt_inner(
        nats,
        wsu,
        workspace_pk,
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
    nats: NatsClient,
    workspace_pk: WorkspacePk,
}

async fn start_server(
    addr: &str,
    bcast: Arc<BroadcastGroup>,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let addr = SocketAddr::from_str(addr)?;

    let workspace_pk = WorkspacePk::generate();

    let mut map = HashMap::new();
    map.insert(format!("{workspace_pk}-my-room"), bcast);

    let (shutdown_broadcast_tx, _shutdown_broadcast_rx) = broadcast::channel(1);

    let mut config = NatsConfig::default();

    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = std::env::var("SI_TEST_NATS_URL") {
        config.url = value;
    }
    let nats = NatsClient::new(&config).await?;
    let state = AppState {
        broadcast_groups: Arc::new(Mutex::new(map)),
        shutdown_broadcast: ShutdownBroadcast::new(shutdown_broadcast_tx),
        nats,
        workspace_pk,
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

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let sink = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = ready!(sink.poll_flush(cx));
        match result {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
