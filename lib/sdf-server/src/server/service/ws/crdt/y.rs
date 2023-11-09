/// Adapted from: https://github.com/y-crdt/yrs-warp/blob/14a1abdf9085d71b6071e27c3e53ac5d0e07735d/src/ws.rs
use axum::extract::ws::{Message, WebSocket};
use futures::stream::{SplitSink, SplitStream};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::RwLock;
use y_sync::awareness::Awareness;
use y_sync::net::Connection;
use y_sync::sync::Error;

/// Connection Wrapper over a [WebSocket], which implements a Yjs/Yrs awareness and update exchange
/// protocol.
///
/// This connection implements Future pattern and can be awaited upon in order for a caller to
/// recognize whether underlying websocket connection has been finished gracefully or abruptly.
#[repr(transparent)]
#[derive(Debug)]
pub struct WarpConn(Connection<WarpSink, WarpStream>);

impl WarpConn {
    pub fn new(awareness: Arc<RwLock<Awareness>>, socket: WebSocket) -> Self {
        let (sink, stream) = socket.split();
        let conn = Connection::new(awareness, WarpSink(sink), WarpStream(stream));
        WarpConn(conn)
    }
}

impl core::future::Future for WarpConn {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
        }
    }
}

/// A warp websocket sink wrapper, that implements futures `Sink` in a way, that makes it compatible
/// with y-sync protocol, so that it can be used by y-sync crate [BroadcastGroup].
///
/// # Examples
///
/// ```rust
/// use std::net::SocketAddr;
/// use std::str::FromStr;
/// use std::sync::Arc;
/// use futures::StreamExt;
/// use tokio::sync::Mutex;
/// use tokio::task::JoinHandle;
/// use warp::{Filter, Rejection, Reply};
/// use warp::ws::{WebSocket, Ws};
/// use yrs_warp::BroadcastGroup;
/// use yrs_warp::ws::{WarpSink, WarpStream};
///
/// async fn start_server(
///     addr: &str,
///     bcast: Arc<BroadcastGroup>,
/// ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
///     let addr = SocketAddr::from_str(addr)?;
///     let ws = warp::path("my-room")
///         .and(warp::ws())
///         .and(warp::any().map(move || bcast.clone()))
///         .and_then(ws_handler);
///
///     Ok(tokio::spawn(async move {
///         warp::serve(ws).run(addr).await;
///     }))
/// }
///
/// async fn ws_handler(ws: Ws, bcast: Arc<BroadcastGroup>) -> Result<impl Reply, Rejection> {
///     Ok(ws.on_upgrade(move |socket| peer(socket, bcast)))
/// }
///
/// async fn peer(ws: WebSocket, bcast: Arc<BroadcastGroup>) {
///     let (sink, stream) = ws.split();
///     // convert warp web socket into compatible sink/stream
///     let sink = Arc::new(Mutex::new(WarpSink::from(sink)));
///     let stream = WarpStream::from(stream);
///     // subscribe to broadcast group
///     let sub = bcast.subscribe(sink, stream);
///     // wait for subscribed connection to close itself
///     match sub.completed().await {
///         Ok(_) => println!("broadcasting for channel finished successfully"),
///         Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
///     }
/// }
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct WarpSink(SplitSink<WebSocket, Message>);

impl From<SplitSink<WebSocket, Message>> for WarpSink {
    fn from(sink: SplitSink<WebSocket, Message>) -> Self {
        WarpSink(sink)
    }
}

impl From<WarpSink> for SplitSink<WebSocket, Message> {
    fn from(value: WarpSink) -> Self {
        value.0
    }
}

impl futures::Sink<Vec<u8>> for WarpSink {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        if let Err(e) = Pin::new(&mut self.0).start_send(Message::Binary(item)) {
            Err(Error::Other(e.into()))
        } else {
            Ok(())
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_flush(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_close(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }
}

/// A warp websocket stream wrapper, that implements futures `Stream` in a way, that makes it compatible
/// with y-sync protocol, so that it can be used by y-sync crate [BroadcastGroup].
///
/// # Examples
///
/// ```rust
/// use std::net::SocketAddr;
/// use std::str::FromStr;
/// use std::sync::Arc;
/// use futures::StreamExt;
/// use tokio::sync::Mutex;
/// use tokio::task::JoinHandle;
/// use warp::{Filter, Rejection, Reply};
/// use warp::ws::{WebSocket, Ws};
/// use yrs_warp::BroadcastGroup;
/// use yrs_warp::ws::{WarpSink, WarpStream};
///
/// async fn start_server(
///     addr: &str,
///     bcast: Arc<BroadcastGroup>,
/// ) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
///     let addr = SocketAddr::from_str(addr)?;
///     let ws = warp::path("my-room")
///         .and(warp::ws())
///         .and(warp::any().map(move || bcast.clone()))
///         .and_then(ws_handler);
///
///     Ok(tokio::spawn(async move {
///         warp::serve(ws).run(addr).await;
///     }))
/// }
///
/// async fn ws_handler(ws: Ws, bcast: Arc<BroadcastGroup>) -> Result<impl Reply, Rejection> {
///     Ok(ws.on_upgrade(move |socket| peer(socket, bcast)))
/// }
///
/// async fn peer(ws: WebSocket, bcast: Arc<BroadcastGroup>) {
///     let (sink, stream) = ws.split();
///     // convert warp web socket into compatible sink/stream
///     let sink = Arc::new(Mutex::new(WarpSink::from(sink)));
///     let stream = WarpStream::from(stream);
///     // subscribe to broadcast group
///     let sub = bcast.subscribe(sink, stream);
///     // wait for subscribed connection to close itself
///     match sub.completed().await {
///         Ok(_) => println!("broadcasting for channel finished successfully"),
///         Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
///     }
/// }
/// ```
#[derive(Debug)]
pub struct WarpStream(SplitStream<WebSocket>);

impl From<SplitStream<WebSocket>> for WarpStream {
    fn from(stream: SplitStream<WebSocket>) -> Self {
        WarpStream(stream)
    }
}

impl From<WarpStream> for SplitStream<WebSocket> {
    fn from(value: WarpStream) -> Self {
        value.0
    }
}

impl Stream for WarpStream {
    type Item = Result<Vec<u8>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(res)) => match res {
                Ok(item) => Poll::Ready(Some(Ok(item.into_data()))),
                Err(e) => Poll::Ready(Some(Err(Error::Other(e.into())))),
            },
        }
    }
}
