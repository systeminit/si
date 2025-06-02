use axum::{
    extract::rejection::JsonRejection,
    response::Json,
};
use dal::{
    WsEvent,
    change_set::ChangeSet,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    api_types::change_sets::v1::ChangeSetViewV1,
    extract::{
        PosthogEventTracker,
        workspace::WorkspaceDalContext,
    },
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "change_sets",
    summary = "Create a Change Set",
    request_body = CreateChangeSetV1Request,
    responses(
        (status = 200, description = "Change Set created successfully", body = CreateChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_change_set(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateChangeSetV1Request>, JsonRejection>,
) -> ChangeSetResult<Json<CreateChangeSetV1Response>> {
    let Json(payload) = payload?;
    let change_set = ChangeSet::fork_head(ctx, &payload.change_set_name).await?;

    let view = ChangeSetViewV1 {
        id: change_set.id,
        name: change_set.clone().name,
        status: change_set.status,
        is_head: change_set.is_head(ctx).await?,
    };

    tracker.track(ctx, "api_create_change_set", json!(payload));

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, payload.change_set_name)
        .await?;

    WsEvent::change_set_created(ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetV1Response { change_set: view }))
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Request {
    #[schema(example = "My new feature", required = true)]
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Response {
    #[schema(example = json!({
        "id": "01FXNV4P306V3KGZ73YSVN8A60",
        "name": "My new feature",
        "status": "Open",
        "isHead": "false"
    }))]
    pub change_set: ChangeSetViewV1,
}
