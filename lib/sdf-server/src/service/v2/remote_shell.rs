use axum::{
    Json,
    Router,
    extract::{Host, OriginalUri},
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use dal::{
    TransactionsError,
};
use si_db;
use sdf_core::api_error::ApiError;
use sdf_extract::{
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use ulid::Ulid;
use veritech_client::{
    ClientError,
    FunctionResult,
    OutputStream,
    RemoteShellConnectionInfo,
    RemoteShellRequest,
    RemoteShellResultSuccess,
    RemoteShellStatus,
};

use crate::{
    AppState,
    extract::PosthogClient,
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

pub type RemoteShellApiResult<T> = Result<T, RemoteShellApiError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum RemoteShellApiError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("failed to receive output stream")]
    OutputReceiver,
    #[error("serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] ClientError),
}

impl IntoResponse for RemoteShellApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            RemoteShellApiError::VeritechClient(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        ApiError::new(status_code, error_message).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRemoteShellSessionRequest {
    pub image: Option<String>,
    pub working_dir: Option<String>,
    pub env_vars: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRemoteShellSessionResponse {
    pub execution_id: String,
    pub session_id: String,
    pub container_id: String,
    pub connection_info: RemoteShellConnectionInfo,
    pub status: RemoteShellStatus,
    pub message: Option<String>,
}

impl From<RemoteShellResultSuccess> for CreateRemoteShellSessionResponse {
    fn from(result: RemoteShellResultSuccess) -> Self {
        Self {
            execution_id: result.execution_id,
            session_id: result.session_id,
            container_id: result.container_id,
            connection_info: result.connection_info,
            status: result.status,
            message: result.message,
        }
    }
}

#[instrument(
    name = "remote_shell.create_session",
    level = "info",
    skip_all,
)]
pub async fn create_session(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateRemoteShellSessionRequest>,
) -> RemoteShellApiResult<ForceChangeSetResponse<CreateRemoteShellSessionResponse>> {
    let force_change_set_id = dal::ChangeSet::force_new(ctx).await?;
    
    // Generate execution ID
    let execution_id = Ulid::new().to_string();

    // Create the remote shell request
    let remote_shell_request = RemoteShellRequest {
        execution_id: execution_id.clone(),
        image: request.image,
        env_vars: request.env_vars.unwrap_or_default(),
        working_dir: request.working_dir,
    };

    // Create output channel (required by veritech client but we don't need the stream for this API)
    let (output_tx, mut _output_rx) = mpsc::channel::<OutputStream>(32);

    // Execute the remote shell request via veritech
    let workspace_id = ctx.tenancy().workspace_pk()?.to_string();
    let change_set_id = ctx.change_set_id().to_string();

    let function_result = ctx
        .veritech()
        .execute_remote_shell(
            output_tx,
            &remote_shell_request,
            &workspace_id,
            &change_set_id,
        )
        .await?;

    // Process the result
    let response = match function_result {
        FunctionResult::Success(result) => {
            info!(
                execution_id = %execution_id,
                session_id = %result.session_id,
                "remote shell session created successfully"
            );
            
            CreateRemoteShellSessionResponse::from(result)
        }
        FunctionResult::Failure(failure) => {
            let error_message = failure.error().message.clone();
            warn!(
                execution_id = %execution_id,
                error = %error_message,
                "remote shell session creation failed"
            );
            
            CreateRemoteShellSessionResponse {
                execution_id: execution_id.clone(),
                session_id: format!("failed_{}", execution_id),
                container_id: String::new(),
                connection_info: RemoteShellConnectionInfo {
                    nats_subject: String::new(),
                    stdin_subject: String::new(),
                    stdout_subject: String::new(),
                    stderr_subject: String::new(),
                    control_subject: String::new(),
                },
                status: RemoteShellStatus::Error,
                message: Some(error_message),
            }
        }
    };

    // Track the event
    track(
        &posthog_client,
        ctx,
        &original_uri,
        &host_name,
        "create_remote_shell_session",
        serde_json::json!({
            "how": "/remote-shell/create",
            "execution_id": execution_id,
            "status": response.status,
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, response))
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_session))
}