use std::sync::Arc;

use axum::{
    extract::{
        ws::{self, WebSocket},
        Extension, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use hyper::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::{PhantomData, Unpin};
use telemetry::{prelude::*, TelemetryLevel};

use super::{
    extract::LimitRequestGuard,
    routes::{State, WatchKeepalive},
};
use crate::{
    server::{execution, execution::Execution, watch},
    CodeGenerationRequest, CodeGenerationResultSuccess, LivenessStatus, Message,
    QualificationCheckRequest, QualificationCheckResultSuccess, ReadinessStatus,
    ResolverFunctionRequest, ResolverFunctionResultSuccess, ResourceSyncRequest,
    ResourceSyncResultSuccess,
};

#[allow(clippy::unused_async)]
pub async fn liveness() -> (StatusCode, &'static str) {
    (StatusCode::OK, LivenessStatus::Ok.into())
}

#[allow(clippy::unused_async)]
pub async fn readiness() -> Result<&'static str, StatusCode> {
    Ok(ReadinessStatus::Ready.into())
}

#[allow(clippy::unused_async)]
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

#[allow(clippy::unused_async)]
pub async fn ws_execute_ping(
    wsu: WebSocketUpgrade,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    async fn handle_socket(mut socket: WebSocket, _limit_request_guard: LimitRequestGuard) {
        if let Err(ref err) = socket.send(ws::Message::Text("pong".to_string())).await {
            warn!("client disconnected; error={}", err);
        }
        if let Err(ref err) = socket.close().await {
            warn!("server failed to close websocket; error={}", err);
        }
    }

    wsu.on_upgrade(move |socket| handle_socket(socket, limit_request_guard))
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_resolver(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ResolverFunctionRequest> = PhantomData;
        let success: PhantomData<ResolverFunctionResultSuccess> = PhantomData;
        handle_socket(
            socket,
            state,
            telemetry_level.is_debug_or_lower(),
            limit_request_guard,
            "resolverfunction".to_owned(),
            request,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_qualification(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<QualificationCheckRequest> = PhantomData;
        let success: PhantomData<QualificationCheckResultSuccess> = PhantomData;
        handle_socket(
            socket,
            state,
            telemetry_level.is_debug_or_lower(),
            limit_request_guard,
            "qualificationcheck".to_owned(),
            request,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_sync(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ResourceSyncRequest> = PhantomData;
        let success: PhantomData<ResourceSyncResultSuccess> = PhantomData;
        handle_socket(
            socket,
            state,
            telemetry_level.is_debug_or_lower(),
            limit_request_guard,
            "resourceSync".to_owned(),
            request,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_code_generation(
    wsu: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<CodeGenerationRequest> = PhantomData;
        let success: PhantomData<CodeGenerationResultSuccess> = PhantomData;
        handle_socket(
            socket,
            state,
            telemetry_level.is_debug_or_lower(),
            limit_request_guard,
            "codeGeneration".to_owned(),
            request,
            success,
        )
    })
}

async fn handle_socket<
    Request: Serialize + DeserializeOwned + Unpin,
    Success: Serialize + DeserializeOwned + Unpin,
>(
    mut socket: WebSocket,
    state: Arc<State>,
    lang_server_debugging: bool,
    _limit_request_guard: LimitRequestGuard,
    sub_command: String,
    _request_marker: PhantomData<Request>,
    success_marker: PhantomData<Success>,
) {
    let proto = {
        let execution: Execution<Request, Success> =
            execution::execute(state.lang_server_path(), lang_server_debugging, sub_command);
        match execution.start(&mut socket).await {
            Ok(started) => started,
            Err(err) => {
                warn!(error = ?err, "failed to start protocol");
                if let Err(err) =
                    fail_to_process(socket, "failed to start protocol", success_marker).await
                {
                    warn!(error = ?err, kind = std::any::type_name::<Request>(), "failed to fail execute function");
                };
                return;
            }
        }
    };
    let proto = match proto.process(&mut socket).await {
        Ok(processed) => processed,
        Err(err) => {
            warn!(error = ?err, "failed to process protocol");
            if let Err(err) =
                fail_to_process(socket, "failed to process protocol", success_marker).await
            {
                warn!(error = ?err, kind = std::any::type_name::<Request>(), "failed to fail execute function");
            };
            return;
        }
    };
    if let Err(err) = proto.finish(socket).await {
        warn!(error = ?err, "failed to finish protocol");
    }
}

async fn fail_to_process<Success: Serialize>(
    mut socket: WebSocket,
    message: impl Into<String>,
    _success_marker: PhantomData<Success>,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Message::<Success>::fail(message).serialize_to_string()?;
    socket.send(ws::Message::Text(msg)).await?;
    socket.close().await?;
    Ok(())
}
