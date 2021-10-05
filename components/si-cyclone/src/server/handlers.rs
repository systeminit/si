use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use hyper::StatusCode;
use tracing::warn;

#[derive(Debug)]
enum LivenessStatus {
    Ok,
}

impl LivenessStatus {
    fn as_str(&self) -> &'static str {
        match self {
            LivenessStatus::Ok => "ok\n",
        }
    }
}

impl From<LivenessStatus> for &'static str {
    fn from(value: LivenessStatus) -> Self {
        value.as_str()
    }
}

pub async fn liveness() -> (StatusCode, &'static str) {
    (StatusCode::OK, LivenessStatus::Ok.into())
}

#[derive(Debug)]
enum ReadinessStatus {
    Ready,
}

impl ReadinessStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ReadinessStatus::Ready => "ready\n",
        }
    }
}

impl From<ReadinessStatus> for &'static str {
    fn from(value: ReadinessStatus) -> Self {
        value.as_str()
    }
}

pub async fn readiness() -> Result<&'static str, StatusCode> {
    Ok(ReadinessStatus::Ready.into())
}

pub async fn execute_ws_ping(ws: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket) {
        if let Err(ref err) = socket.send(Message::Text("pong".to_string())).await {
            warn!("client disconnected; error={}", err);
        }
    }

    ws.on_upgrade(handle_socket);
}

pub async fn execute_ws_resolver(ws: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(_socket: WebSocket) {
        todo!("HAY resolver!")
    }

    ws.on_upgrade(handle_socket);
}
