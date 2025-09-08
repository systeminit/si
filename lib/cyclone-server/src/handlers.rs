use std::{
    fmt,
    marker::{
        PhantomData,
        Unpin,
    },
    sync::Arc,
};

use axum::{
    extract::{
        Extension,
        State,
        WebSocketUpgrade,
        ws::{
            self,
            WebSocket,
        },
    },
    response::IntoResponse,
};
use cyclone_core::{
    ActionRunRequest,
    ActionRunResultSuccess,
    CycloneRequestable,
    FunctionResult,
    LivenessStatus,
    ManagementRequest,
    ManagementResultSuccess,
    Message,
    ReadinessStatus,
    RemoteShellConnectionInfo,
    RemoteShellRequest,
    RemoteShellResultSuccess,
    RemoteShellStatus,
    ResolverFunctionRequest,
    ResolverFunctionResultSuccess,
    SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess,
    ValidationRequest,
    ValidationResultSuccess,
};
use hyper::StatusCode;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use telemetry::prelude::*;
use telemetry_http::ParentSpan;

use super::extract::LimitRequestGuard;
use crate::{
    execution::{
        self,
        Execution,
    },
    result::{
        LangServerActionRunResultSuccess,
        LangServerResolverFunctionResultSuccess,
        LangServerValidationResultSuccess,
    },
    state::{
        LangServerChild,
        LangServerProcessTimeout,
        WatchKeepalive,
    },
    watch,
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

pub async fn ws_execute_ping(
    wsu: WebSocketUpgrade,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    async fn handle_socket(
        mut socket: WebSocket,
        request_span: Span,
        _limit_request_guard: LimitRequestGuard,
    ) {
        let mut has_errored = false;

        if let Err(ref err) = socket.send(ws::Message::Text("pong".to_string())).await {
            request_span.record_err(err);
            warn!("client disconnected; error={}", err);
            has_errored = true;
        }
        if let Err(ref err) = socket.close().await {
            request_span.record_err(err);
            warn!("server failed to close websocket; error={}", err);
            has_errored = true;
        }

        if !has_errored {
            request_span.record_ok();
        }
    }

    wsu.on_upgrade(move |socket| {
        handle_socket(socket, request_span.into_inner(), limit_request_guard)
    })
}

pub async fn ws_execute_resolver(
    wsu: WebSocketUpgrade,
    State(lang_server_process_timeout): State<LangServerProcessTimeout>,
    State(child): State<LangServerChild>,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ResolverFunctionRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerResolverFunctionResultSuccess> = PhantomData;
        let success: PhantomData<ResolverFunctionResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_process_timeout.inner(),
            limit_request_guard,
            request,
            lang_server_success,
            success,
            request_span.into_inner(),
            child,
        )
    })
}

pub async fn ws_execute_validation(
    wsu: WebSocketUpgrade,
    State(lang_server_process_timeout): State<LangServerProcessTimeout>,
    State(child): State<LangServerChild>,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ValidationRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerValidationResultSuccess> = PhantomData;
        let success: PhantomData<ValidationResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_process_timeout.inner(),
            limit_request_guard,
            request,
            lang_server_success,
            success,
            request_span.into_inner(),
            child,
        )
    })
}

pub async fn ws_execute_action_run(
    wsu: WebSocketUpgrade,
    State(lang_server_process_timeout): State<LangServerProcessTimeout>,
    State(child): State<LangServerChild>,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ActionRunRequest> = PhantomData;
        let lang_server_success: PhantomData<LangServerActionRunResultSuccess> = PhantomData;
        let success: PhantomData<ActionRunResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_process_timeout.inner(),
            limit_request_guard,
            request,
            lang_server_success,
            success,
            request_span.into_inner(),
            child,
        )
    })
}

pub async fn ws_execute_schema_variant_definition(
    wsu: WebSocketUpgrade,
    State(lang_server_process_timeout): State<LangServerProcessTimeout>,
    State(child): State<LangServerChild>,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<SchemaVariantDefinitionRequest> = PhantomData;
        let lang_server_success: PhantomData<SchemaVariantDefinitionResultSuccess> = PhantomData;
        let success: PhantomData<SchemaVariantDefinitionResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_process_timeout.inner(),
            limit_request_guard,
            request,
            lang_server_success,
            success,
            request_span.into_inner(),
            child,
        )
    })
}

pub async fn ws_execute_management(
    wsu: WebSocketUpgrade,
    State(lang_server_process_timeout): State<LangServerProcessTimeout>,
    State(child): State<LangServerChild>,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    wsu.on_upgrade(move |socket| {
        let request: PhantomData<ManagementRequest> = PhantomData;
        let lang_server_success: PhantomData<ManagementResultSuccess> = PhantomData;
        let success: PhantomData<ManagementResultSuccess> = PhantomData;
        handle_socket(
            socket,
            lang_server_process_timeout.inner(),
            limit_request_guard,
            request,
            lang_server_success,
            success,
            request_span.into_inner(),
            child,
        )
    })
}

#[instrument(
    name = "web_socket.handle_socket",
    parent = &request_span,
    level = "info",
    skip_all,
    fields()
)]
#[allow(clippy::too_many_arguments)]
async fn handle_socket<Request, LangServerSuccess, Success>(
    mut socket: WebSocket,
    lang_server_process_timeout: Option<u64>,
    _limit_request_guard: LimitRequestGuard,
    _request_marker: PhantomData<Request>,
    _lang_server_success_marker: PhantomData<LangServerSuccess>,
    success_marker: PhantomData<Success>,
    request_span: Span,
    child: LangServerChild,
) where
    Request: Serialize + DeserializeOwned + Unpin + fmt::Debug + CycloneRequestable,
    Success: Serialize + Unpin + fmt::Debug,
    LangServerSuccess: Serialize + DeserializeOwned + Unpin + fmt::Debug + Into<Success>,
{
    let proto = {
        let execution: Execution<Request, LangServerSuccess, Success> =
            execution::new(lang_server_process_timeout);
        match execution.start(child, &mut socket).await {
            Ok(started) => started,
            Err(err) => {
                warn!(si.error.message = ?err, "failed to start protocol");
                request_span.record_err(&err);
                if let Err(err) =
                    fail_to_process(socket, "failed to start protocol", success_marker).await
                {
                    warn!(
                        error = ?err,
                        kind = std::any::type_name::<Request>(),
                        "failed to fail execute function",
                    );
                };
                return;
            }
        }
    };
    let proto = match proto.process(&mut socket).await {
        Ok(processed) => processed,
        Err(err) => {
            warn!(si.error.message = ?err, "failed to process protocol");
            request_span.record_err(&err);
            if let Err(err) = fail_to_process(
                socket,
                format!("failed to process protocol: {err:?}"),
                success_marker,
            )
            .await
            {
                warn!(
                    error = ?err,
                    kind = std::any::type_name::<Request>(),
                    "failed to fail execute function",
                );
            };
            return;
        }
    };
    if let Err(err) = proto.finish(socket).await {
        request_span.record_err(&err);
        warn!(si.error.message = ?err, "failed to finish protocol");
        return;
    }

    request_span.record_ok();
}

pub async fn ws_execute_remote_shell(
    wsu: WebSocketUpgrade,
    limit_request_guard: LimitRequestGuard,
    Extension(request_span): Extension<ParentSpan>,
) -> impl IntoResponse {
    async fn handle_socket(
        mut socket: WebSocket,
        request_span: Span,
        _limit_request_guard: LimitRequestGuard,
    ) {
        let mut has_errored = false;

        // Read the RemoteShellRequest from the websocket
        let request = match socket.recv().await {
            Some(Ok(ws::Message::Text(text))) => {
                match serde_json::from_str::<RemoteShellRequest>(&text) {
                    Ok(req) => req,
                    Err(err) => {
                        request_span.record_err(&err);
                        warn!("failed to parse remote shell request; error={}", err);
                        has_errored = true;
                        let _ = socket.send(ws::Message::Text(
                            serde_json::to_string(&Message::<RemoteShellResultSuccess>::fail(
                                "failed to parse remote shell request"
                            )).unwrap_or_else(|_| "{}".to_string())
                        )).await;
                        let _ = socket.close().await;
                        return;
                    }
                }
            }
            Some(Ok(_)) => {
                warn!("received non-text message for remote shell request");
                has_errored = true;
                let _ = socket.send(ws::Message::Text(
                    serde_json::to_string(&Message::<RemoteShellResultSuccess>::fail(
                        "expected text message"
                    )).unwrap_or_else(|_| "{}".to_string())
                )).await;
                let _ = socket.close().await;
                return;
            }
            Some(Err(err)) => {
                request_span.record_err(&err);
                warn!("websocket error receiving remote shell request; error={}", err);
                has_errored = true;
                let _ = socket.close().await;
                return;
            }
            None => {
                warn!("websocket closed before receiving remote shell request");
                has_errored = true;
                return;
            }
        };

        // For now, return a placeholder response indicating that remote shell
        // functionality is not yet fully implemented
        let result = RemoteShellResultSuccess {
            execution_id: request.execution_id.clone(),
            session_id: format!("session_{}", request.execution_id),
            container_id: format!("container_{}", request.execution_id),
            connection_info: RemoteShellConnectionInfo {
                nats_subject: format!("remote_shell.{}.control", request.execution_id),
                stdin_subject: format!("remote_shell.{}.stdin", request.execution_id),
                stdout_subject: format!("remote_shell.{}.stdout", request.execution_id),
                stderr_subject: format!("remote_shell.{}.stderr", request.execution_id),
                control_subject: format!("remote_shell.{}.control", request.execution_id),
            },
            status: RemoteShellStatus::Active,
            message: Some("Remote shell session placeholder - not yet fully implemented".to_string()),
        };

        let response_message = Message::Result(FunctionResult::Success(result));
        let response_text = match serde_json::to_string(&response_message) {
            Ok(text) => text,
            Err(err) => {
                request_span.record_err(&err);
                warn!("failed to serialize remote shell response; error={}", err);
                has_errored = true;
                let _ = socket.close().await;
                return;
            }
        };

        if let Err(ref err) = socket.send(ws::Message::Text(response_text)).await {
            request_span.record_err(err);
            warn!("client disconnected; error={}", err);
            has_errored = true;
        }
        if let Err(ref err) = socket.close().await {
            request_span.record_err(err);
            warn!("server failed to close websocket; error={}", err);
            has_errored = true;
        }

        if !has_errored {
            request_span.record_ok();
        }
    }

    wsu.on_upgrade(move |socket| {
        handle_socket(socket, request_span.into_inner(), limit_request_guard)
    })
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
