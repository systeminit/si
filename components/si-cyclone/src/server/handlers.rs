use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Extension, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::StreamExt;
use hyper::StatusCode;
use tracing::warn;

use super::routes::State;
use crate::{server::resolver_function, LivenessStatus, ReadinessStatus};

pub async fn liveness() -> (StatusCode, &'static str) {
    (StatusCode::OK, LivenessStatus::Ok.into())
}

pub async fn readiness() -> Result<&'static str, StatusCode> {
    Ok(ReadinessStatus::Ready.into())
}

pub async fn ws_execute_ping(wsu: WebSocketUpgrade) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket) {
        if let Err(ref err) = socket.send(Message::Text("pong".to_string())).await {
            warn!("client disconnected; error={}", err);
        }
    }

    wsu.on_upgrade(handle_socket)
}

pub async fn ws_execute_resolver(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    async fn handle_socket(socket: WebSocket, state: Arc<State>) {
        let mut progress = match resolver_function::execute(socket, state.lang_server_path())
            .start()
            .await
        {
            Ok(progress) => progress,
            Err(err) => panic!("starting failed: {:?}", err),
        };
        while let Some(message) = progress.next().await {
            dbg!(message);
        }
        match progress.finish().await {
            Ok(_) => todo!(),
            Err(err) => panic!("failed to finish: {:?}", err),
        }
    }

    wsu.on_upgrade(move |socket| handle_socket(socket, state))
}
