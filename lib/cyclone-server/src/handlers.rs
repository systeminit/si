use std::{
    fmt,
    marker::{PhantomData, Unpin},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{
        ws::{self, WebSocket},
        Extension, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use cyclone_core::{
    CommandRunRequest, CommandRunResultSuccess, LivenessStatus, Message, ReadinessStatus,
    ResolverFunctionRequest, ResolverFunctionResultSuccess, ValidationRequest,
    ValidationResultSuccess, WorkflowResolveRequest, WorkflowResolveResultSuccess,
};
use hyper::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use telemetry::{prelude::*, TelemetryLevel};
use tokio::time;

use super::{
    extract::LimitRequestGuard,
    routes::{LangServerPath, WatchKeepalive},
};
use crate::result::LangServerValidationResultSuccess;
use crate::{
    execution::{self, Execution},
    request::{DecryptRequest, ListSecrets},
    result::{
        LangServerCommandRunResultSuccess, LangServerResolverFunctionResultSuccess,
        LangServerWorkflowResolveResultSuccess,
    },
    watch, DecryptionKey,
};

const PROCESS_TIMEOUT: Duration = Duration::from_secs(25);

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
    Extension(lang_server_path): Extension<Arc<LangServerPath>>,
    Extension(key): Extension<Arc<DecryptionKey>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    let lang_server_path = lang_server_path.as_path().to_path_buf();
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ResolverFunctionRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerResolverFunctionResultSuccess> = PhantomData;
        let success: PhantomData<ResolverFunctionResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_path,
            telemetry_level.is_debug_or_lower(),
            key,
            limit_request_guard,
            "resolverfunction".to_owned(),
            request,
            lang_server_success,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_validation(
    wsu: WebSocketUpgrade,
    Extension(lang_server_path): Extension<Arc<LangServerPath>>,
    Extension(key): Extension<Arc<DecryptionKey>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    let lang_server_path = lang_server_path.as_path().to_path_buf();
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ValidationRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerValidationResultSuccess> = PhantomData;
        let success: PhantomData<ValidationResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_path,
            telemetry_level.is_debug_or_lower(),
            key,
            limit_request_guard,
            "validation".to_owned(),
            request,
            lang_server_success,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_workflow_resolve(
    wsu: WebSocketUpgrade,
    Extension(lang_server_path): Extension<Arc<LangServerPath>>,
    Extension(key): Extension<Arc<DecryptionKey>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    let lang_server_path = lang_server_path.as_path().to_path_buf();
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<WorkflowResolveRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerWorkflowResolveResultSuccess> = PhantomData;
        let success: PhantomData<WorkflowResolveResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_path,
            telemetry_level.is_debug_or_lower(),
            key,
            limit_request_guard,
            "workflowResolve".to_owned(),
            request,
            lang_server_success,
            success,
        )
    })
}

#[allow(clippy::unused_async)]
pub async fn ws_execute_command_run(
    wsu: WebSocketUpgrade,
    Extension(lang_server_path): Extension<Arc<LangServerPath>>,
    Extension(key): Extension<Arc<DecryptionKey>>,
    Extension(telemetry_level): Extension<Arc<Box<dyn TelemetryLevel>>>,
    limit_request_guard: LimitRequestGuard,
) -> impl IntoResponse {
    let lang_server_path = lang_server_path.as_path().to_path_buf();

    // Hey! Let's upgrade from http communication to websocket communication.
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<CommandRunRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerCommandRunResultSuccess> = PhantomData;
        let success: PhantomData<CommandRunResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_path,
            telemetry_level.is_debug_or_lower(),
            key,
            limit_request_guard,
            "commandRun".to_owned(),
            request,
            lang_server_success,
            success,
        )
    })
}

#[allow(clippy::too_many_arguments)]
async fn handle_socket<Request, LangServerSuccess, Success>(
    mut socket: WebSocket,
    lang_server_path: PathBuf,
    lang_server_debugging: bool,
    key: Arc<DecryptionKey>,
    _limit_request_guard: LimitRequestGuard,
    sub_command: String,
    _request_marker: PhantomData<Request>,
    _lang_server_success_marker: PhantomData<LangServerSuccess>,
    success_marker: PhantomData<Success>,
) where
    Request: DecryptRequest + ListSecrets + Serialize + DeserializeOwned + Unpin + fmt::Debug,
    Success: Serialize + Unpin + fmt::Debug,
    LangServerSuccess: Serialize + DeserializeOwned + Unpin + fmt::Debug + Into<Success>,
{
    let proto = {
        let execution: Execution<Request, LangServerSuccess, Success> =
            execution::new(lang_server_path, lang_server_debugging, key, sub_command);
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
    let proto = match time::timeout(PROCESS_TIMEOUT, proto.process(&mut socket)).await {
        // An execution was successfully completed and returned under the timeout limit
        Ok(Ok(processed)) => processed,
        // An execution failed to complete and return under the timeout limit
        Ok(Err(err)) => {
            warn!(error = ?err, "failed to process protocol");
            if let Err(err) = fail_to_process(
                socket,
                format!("failed to process protocol: {err:?}"),
                success_marker,
            )
            .await
            {
                warn!(error = ?err, kind = std::any::type_name::<Request>(), "failed to fail execute function");
            };
            return;
        }
        // The timeout limit was exceeded
        Err(_elapsed) => {
            warn!(
                kind = std::any::type_name::<Request>(),
                timeout = PROCESS_TIMEOUT.as_secs(),
                "function execution timeout exceeded"
            );
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
