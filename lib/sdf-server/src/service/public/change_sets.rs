use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dal::{change_set::ChangeSet, WsEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use thiserror::Error;

use crate::extract::{workspace::WorkspaceDalContext, PosthogEventTracker};
use crate::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetsError {
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

type Result<T> = std::result::Result<T, ChangeSetsError>;

impl IntoResponse for ChangeSetsError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

// /api/public/workspaces/:workspace_id/change-sets
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_change_set))
        .nest("/:change_set_id", Router::new())
}

async fn create_change_set(
    WorkspaceDalContext(ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    Json(payload): Json<CreateChangeSetRequest>,
) -> Result<Json<CreateChangeSetResponse>> {
    let change_set = ChangeSet::fork_head(&ctx, &payload.change_set_name).await?;

    tracker.track(&ctx, "create_change_set", json!(payload));

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, payload.change_set_name)
        .await?;

    WsEvent::change_set_created(&ctx, change_set.id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateChangeSetRequest {
    change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateChangeSetResponse {
    change_set: ChangeSet,
}
