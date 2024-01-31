/// Adapted from: https://github.com/y-crdt/yrs-warp/blob/14a1abdf9085d71b6071e27c3e53ac5d0e07735d/src/ws.rs
use axum::extract::ws::Message;
use dal::WorkspacePk;
use futures::{Future, Sink, SinkExt, Stream};
use futures_lite::future::FutureExt;
use nats_multiplexer::Multiplexer;
use nats_multiplexer_client::MultiplexerClient;
use sdf_server::server::service::ws::crdt::{crdt_handle, BroadcastGroups, CrdtError};
use sdf_server::server::CRDT_MULTIPLEXER_SUBJECT;
use si_data_nats::{NatsClient, NatsConfig, Subject};
use std::{collections::HashMap, pin::Pin, sync::Arc, task::Context, task::Poll, time::Duration};
use tokio::{
    sync::broadcast, sync::Mutex, sync::Notify, sync::RwLock, task, task::JoinHandle, time::timeout,
};
use y_sync::{awareness::Awareness, net::BroadcastGroup, net::Connection};
use yrs::{updates::encoder::Encode, Doc, GetString, Text, Transact, UpdateSubscription};

struct Server {
    nats: NatsClient,
    channel_name: Subject,
    workspace_pk: WorkspacePk,
    id: String,
    broadcast_groups: BroadcastGroups,
    crdt_multiplexer_client: MultiplexerClient,
    _shutdown_broadcast_tx: broadcast::Sender<()>,
    _shutdown_broadcast_rx: broadcast::Receiver<()>,
}

struct Client {
    _handle: JoinHandle<()>,
    conn: Connection<TestSink, TestStream>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown_broadcast_tx
            .send(())
            .expect("unable to drop client");
    }
}

async fn client(doc: Doc, server: &Server) -> Result<Client, Box<dyn std::error::Error>> {
    let (sink, stream) = broadcast::channel(1000000);
    let sink = TestWsSink::new(sink);
    let stream = TestWsStream::new(stream);

    let receiver = server
        .crdt_multiplexer_client
        .receiver(server.channel_name.clone())
        .await?;
    let ws_receiver = receiver.resubscribe();

    let (shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let _handle = tokio::spawn(crdt_handle(
        sink.clone(),
        stream.clone(),
        server.nats.clone(),
        server.broadcast_groups.clone(),
        server.channel_name.clone(),
        receiver,
        ws_receiver,
        server.workspace_pk,
        server.id.clone(),
        shutdown_broadcast_rx,
    ));

    let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
    let conn = Connection::new(awareness, TestSink::new(sink), TestStream::new(stream));
    Ok(Client {
        _handle,
        conn,
        shutdown_broadcast_tx,
    })
}

async fn start_server(
    awareness: Arc<RwLock<Awareness>>,
) -> Result<Server, Box<dyn std::error::Error>> {
    let id = "my-room".to_owned();
    let bcast = Arc::new(BroadcastGroup::new(awareness.clone(), 10).await);

    let workspace_pk = WorkspacePk::generate();

    let mut map = HashMap::new();
    map.insert(format!("{workspace_pk}-{id}"), bcast);
    let broadcast_groups = Arc::new(Mutex::new(map));

    let mut config = NatsConfig::default();

    #[allow(clippy::disallowed_methods)] // Used only in tests & so prefixed with `SI_TEST_`
    if let Ok(value) = std::env::var("SI_TEST_NATS_URL") {
        config.url = value;
    }
    let nats = NatsClient::new(&config).await?;

    let channel_name = format!("crdt.{workspace_pk}.{id}").into();

    // NOTE(nick,paulo,fletcher): we need to ensure the lifetimes of these correspond to the lifetime of an entire test.
    let (_shutdown_broadcast_tx, shutdown_broadcast_rx) = broadcast::channel(1);

    let (crdt_multiplexer, crdt_multiplexer_client) =
        Multiplexer::new(&nats, CRDT_MULTIPLEXER_SUBJECT).await?;

    tokio::spawn(crdt_multiplexer.run(shutdown_broadcast_rx.resubscribe()));

    Ok(Server {
        nats,
        broadcast_groups,
        channel_name,
        workspace_pk,
        id,
        crdt_multiplexer_client,
        _shutdown_broadcast_tx,
        _shutdown_broadcast_rx: shutdown_broadcast_rx,
    })
}

#[derive(Clone)]
struct TestSink(TestWsSink);

impl TestSink {
    pub fn new(sink: TestWsSink) -> Self {
        Self(sink)
    }
}

impl Sink<Vec<u8>> for TestSink {
    type Error = y_sync::sync::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_ready(cx) {
            Poll::Ready(Ok(msg)) => Poll::Ready(Ok(msg)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(y_sync::sync::Error::Other(err.into()))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn start_send(mut self: Pin<&mut Self>, message: Vec<u8>) -> Result<(), Self::Error> {
        match Pin::new(&mut self.0).start_send(Message::Binary(message)) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(y_sync::sync::Error::Other(err.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_flush(cx) {
            Poll::Ready(Ok(msg)) => Poll::Ready(Ok(msg)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(y_sync::sync::Error::Other(err.into()))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_close(cx) {
            Poll::Ready(Ok(msg)) => Poll::Ready(Ok(msg)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(y_sync::sync::Error::Other(err.into()))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone)]
struct TestStream(TestWsStream);

impl TestStream {
    pub fn new(stream: TestWsStream) -> Self {
        Self(stream)
    }
}

impl Stream for TestStream {
    type Item = Result<Vec<u8>, y_sync::sync::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Ready(Some(Ok(Message::Binary(msg)))) => Poll::Ready(Some(Ok(msg))),
            Poll::Ready(Some(Ok(_))) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(y_sync::sync::Error::Other(err.into()))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

struct TestWsSink {
    id: u64,
    sink: Option<broadcast::Sender<Message>>,
}

impl TestWsSink {
    pub fn new(sender: broadcast::Sender<Message>) -> Self {
        Self {
            id: 0,
            sink: Some(sender),
        }
    }
}

impl Clone for TestWsSink {
    fn clone(&self) -> Self {
        Self {
            id: self.id + 1,
            sink: self.sink.clone(),
        }
    }
}

impl Sink<Message> for TestWsSink {
    type Error = CrdtError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, payload: Message) -> Result<(), Self::Error> {
        if let Some(sink) = &mut self.sink {
            sink.send(payload)?;
        }
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // Drops it which triggers the channel closure
        self.sink.take();
        Poll::Ready(Ok(()))
    }
}

type BoxedResultFuture<T, E> = Box<dyn Future<Output = Result<T, E>> + Sync + Send>;

struct TestWsStream {
    id: u64,
    stream: broadcast::Receiver<Message>,
    future: Option<Pin<BoxedResultFuture<Message, axum::Error>>>,
}

impl Clone for TestWsStream {
    fn clone(&self) -> Self {
        Self {
            id: self.id + 1,
            stream: self.stream.resubscribe(),
            future: None,
        }
    }
}

impl TestWsStream {
    pub fn new(stream: broadcast::Receiver<Message>) -> Self {
        Self {
            id: 0,
            stream,
            future: None,
        }
    }
}

impl Stream for TestWsStream {
    type Item = Result<Message, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.future.is_none() {
            let mut stream = self.stream.resubscribe();
            let value = match self.stream.try_recv() {
                Ok(msg) => Some(msg),
                Err(broadcast::error::TryRecvError::Empty) => None,
                Err(err @ broadcast::error::TryRecvError::Closed) => {
                    return Poll::Ready(Some(Err(axum::Error::new(err))))
                }
                Err(broadcast::error::TryRecvError::Lagged(num)) => {
                    panic!("Broadcast reader lagged behind {} messages", num)
                }
            };

            if let Some(value) = value {
                return Poll::Ready(Some(Ok(value)));
            } else {
                self.future = Some(Box::pin(async move {
                    stream.recv().await.map_err(axum::Error::new)
                }));
            }
        }

        if let Some(mut future) = self.future.take() {
            match future.poll(cx) {
                Poll::Ready(msg) => Poll::Ready(Some(msg)),
                Poll::Pending => {
                    self.future = Some(future);
                    Poll::Pending
                }
            }
        } else {
            Poll::Pending
        }
    }
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
    let server = start_server(awareness.clone()).await?;

    let doc = Doc::new();
    let (n, _sub) = create_notifier(&doc);
    let c1 = client(doc, &server).await.expect("unable to make client");

    {
        let lock = awareness.write().await;
        text.push(&mut lock.doc().transact_mut(), "abc");
    }

    timeout(TIMEOUT, n.notified()).await?;

    {
        let awareness = c1.conn.awareness().read().await;
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
    let server = start_server(awareness).await?;

    let doc = Doc::new();
    let (n, _sub) = create_notifier(&doc);
    let c1 = client(doc, &server).await.expect("unable to make client");

    timeout(TIMEOUT, n.notified()).await?;

    {
        let awareness = c1.conn.awareness().read().await;
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
    let server = start_server(awareness).await?;

    let d1 = Doc::with_client_id(2);
    let c1 = client(d1, &server).await.expect("unable to make client");

    // by default changes made by document on the client side are not propagated automatically
    let _sub11 = {
        let sink = c1.conn.sink();
        let a = c1.conn.awareness().write().await;
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
    let c2 = client(d2, &server).await.expect("unable to make client");

    {
        let a = c1.conn.awareness().write().await;
        let doc = a.doc();
        let text = doc.get_or_insert_text("test");
        text.push(&mut doc.transact_mut(), "def");
    }

    timeout(TIMEOUT, n2.notified()).await?;

    {
        let awareness = c2.conn.awareness().read().await;
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

    let server = start_server(awareness).await?;

    let d1 = Doc::with_client_id(2);
    let c1 = client(d1, &server).await.expect("unable to make client");

    // by default changes made by document on the client side are not propagated automatically
    let _sub11 = {
        let sink = c1.conn.sink();
        let a = c1.conn.awareness().write().await;
        let doc = a.doc();
        doc.observe_update_v1(move |_txn, e| {
            let update = e.update.to_owned();
            if let Some(sink) = sink.upgrade() {
                task::spawn(async move {
                    let msg =
                        y_sync::sync::Message::Sync(y_sync::sync::SyncMessage::Update(update))
                            .encode_v1();
                    let mut sink = sink.lock().await;
                    sink.send(dbg!(msg))
                        .await
                        .expect("unable to send msg to sink");
                });
            }
        })
        .expect("unable to observe update v1")
    };

    let d2 = Doc::with_client_id(3);
    let (n2, sub2) = create_notifier(&d2);
    let c2 = client(d2, &server).await.expect("unable to make client");

    let d3 = Doc::with_client_id(4);
    let (n3, sub3) = create_notifier(&d3);
    let c3 = client(d3, &server).await.expect("unable to make client");

    {
        let a = c1.conn.awareness().write().await;
        let doc = a.doc();
        let text = doc.get_or_insert_text("test");
        text.push(&mut doc.transact_mut(), "abc");
    }

    // on the first try both C2 and C3 should receive the update
    timeout(TIMEOUT, n2.notified()).await.unwrap();
    // timeout(TIMEOUT, n2.notified()).await.unwrap();

    {
        let awareness = c2.conn.awareness().read().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("test");
        let str = text.get_string(&doc.transact());
        assert_eq!(str, "abc".to_string());
    }

    {
        let awareness = c3.conn.awareness().read().await;
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
        let a = c2.conn.awareness().write().await;
        let doc = a.doc();
        create_notifier(doc)
    };

    {
        let a = c1.conn.awareness().write().await;
        let doc = a.doc();
        let text = doc.get_or_insert_text("test");
        text.push(&mut doc.transact_mut(), "def");
    }

    timeout(TIMEOUT, n2.notified()).await.expect("timeout");

    {
        let awareness = c2.conn.awareness().read().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("test");
        let str = text.get_string(&doc.transact());
        assert_eq!(str, "abcdef".to_string());
    }

    Ok(())
}
