use axum::response::Json;
use dal::change_set::ChangeSet;
use dal::WsEvent;
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::ToSchema;

use crate::extract::{workspace::WorkspaceDalContext, PosthogEventTracker};

use crate::service::v1::ChangeSetError;

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets",
    params(
        ("workspace_id", description = "Workspace identifier")
    ),
    tag = "change_sets",
    request_body = CreateChangeSetV1Request,
    responses(
        (status = 200, description = "Change set created successfully", body = CreateChangeSetV1Response),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_change_set(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    Json(payload): Json<CreateChangeSetV1Request>,
) -> Result<Json<CreateChangeSetV1Response>, ChangeSetError> {
    let change_set = ChangeSet::fork_head(ctx, &payload.change_set_name).await?;

    tracker.track(ctx, "api_create_change_set", json!(payload));

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, payload.change_set_name)
        .await?;

    WsEvent::change_set_created(ctx, change_set.id)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetV1Response { change_set }))
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Request {
    #[schema(example = "My new feature")]
    pub change_set_name: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Response {
    #[schema(value_type = Object, example = json!({"id": "01FXNV4P306V3KGZ73YSVN8A60", "name": "My new feature"}))]
    pub change_set: ChangeSet,
}
