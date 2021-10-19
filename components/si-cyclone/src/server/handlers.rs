use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Extension, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use hyper::StatusCode;
use tracing::{trace, warn};

use super::routes::{State, WatchKeepalive};
use crate::{
    resolver_function::ResolverFunctionMessage,
    server::{resolver_function, watch},
    LivenessStatus, ReadinessStatus,
};

#[allow(clippy::unused_async)]
pub async fn liveness() -> (StatusCode, &'static str) {
    (StatusCode::OK, LivenessStatus::Ok.into())
}

#[allow(clippy::unused_async)]
pub async fn readiness() -> Result<&'static str, StatusCode> {
    Ok(ReadinessStatus::Ready.into())
}

pub async fn ws_watch(
    wsu: WebSocketUpgrade,
    Extension(watch_keepalive): Extension<Arc<WatchKeepalive>>,
) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket, watch_keepalive: Arc<WatchKeepalive>) {
        if let Err(err) = watch::run(watch_keepalive.clone_tx(), watch_keepalive.timeout())
            .start(&mut socket)
            .await
        {
            // An error is most likely returned when the client side terminates the websocket
            // session or if a network partition occurs, so this is our "normal" behavior
            trace!(error = ?err, "protocol finished");
        }
    }

    wsu.on_upgrade(move |socket| handle_socket(socket, watch_keepalive))
}

pub async fn ws_execute_ping(wsu: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket) {
        if let Err(ref err) = socket.send(Message::Text("pong".to_string())).await {
            warn!("client disconnected; error={}", err);
        }
    }

    wsu.on_upgrade(handle_socket)
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_resolver(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket, state: Arc<State>) {
        let proto = match resolver_function::execute(state.lang_server_path())
            .start(&mut socket)
            .await
        {
            Ok(started) => started,
            Err(err) => {
                warn!(error = ?err, "failed to start protocol");
                if let Err(err) = fail_execute_resolver(socket, "failed to start protocol").await {
                    warn!(error = ?err, "failed to fail execute resolver");
                };
                return;
            }
        };
        let proto = match proto.process(&mut socket).await {
            Ok(processed) => processed,
            Err(err) => {
                warn!(error = ?err, "failed to process protocol");
                if let Err(err) = fail_execute_resolver(socket, "failed to process protocol").await
                {
                    warn!(error = ?err, "failed to fail execute resolver");
                };
                return;
            }
        };
        if let Err(err) = proto.finish(socket).await {
            warn!(error = ?err, "failed to finish protocol");
        }
    }

    wsu.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn fail_execute_resolver(
    mut socket: WebSocket,
    message: impl Into<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = ResolverFunctionMessage::fail(message).serialize_to_string()?;
    socket.send(Message::Text(msg)).await?;
    socket.close().await?;
    Ok(())
}
