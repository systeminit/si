use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use dal::WsEvent;
use sdf_core::EddaClientError;
use sdf_extract::workspace::WorkspaceDalContext;
use si_db::HistoryActor;
use thiserror::Error;

use crate::service::v1::common::ErrorIntoResponse;

pub type UserResult<T> = Result<T, UserError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum UserError {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("db error: {0}")]
    Db(#[from] si_db::Error),
    #[error("edda client error: {0}")]
    EddaClient(#[from] EddaClientError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("missing user on request")]
    MissingUser,
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

impl ErrorIntoResponse for UserError {
    fn status_and_message(&self) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
    }
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        self.to_api_response()
    }
}

pub enum UserResultV1 {
    Empty
}

impl IntoResponse for UserResultV1 {
    fn into_response(self) -> Response {
        match self {
            Self::Empty => {
                StatusCode::NO_CONTENT.into_response()
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/user/set_ai_agent_executed",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "user",
    responses(
        (status = 204, description = "Flag set successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn set_ai_agent_executed(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
) -> UserResult<UserResultV1> {
    let HistoryActor::User(user_pk) = ctx.history_actor() else {
        return Err(UserError::MissingUser);
    };

    let flags = si_db::User::set_flag_for_user_on_workspace(
        ctx,
        *user_pk,
        ctx.workspace_pk()?,
        "executedAgent",
        serde_json::Value::Bool(true),
    )
    .await?;

    WsEvent::user_workspace_flags_update(ctx.workspace_pk()?, *user_pk, flags)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;

    Ok(UserResultV1::Empty)
}
