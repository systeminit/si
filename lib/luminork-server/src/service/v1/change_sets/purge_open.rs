use axum::response::Json;
use dal::{
    ChangeSetId,
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
use crate::extract::{
    PosthogEventTracker,
    workspace::WorkspaceDalContext,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/purge_open",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "change_sets",
    summary = "Abandon all active Change Sets",
    responses(
        (status = 200, description = "Change Sets purged successfully", body = PurgeOpenChangeSetsV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn purge_open(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<PurgeOpenChangeSetsV1Response>> {
    let mut change_set_ids: Vec<ChangeSetId> = vec![];
    let open_change_sets = ChangeSet::list_active(ctx).await?;
    let head = ctx.get_workspace_default_change_set_id().await?;
    for mut change_set in open_change_sets {
        if change_set.id == head {
            continue;
        }
        change_set_ids.push(change_set.id);
        change_set.abandon(ctx).await?;
    }

    tracker.track(
        ctx,
        "api_purge_open_change_sets",
        json!({
            "change_set_ids": change_set_ids
        }),
    );

    ctx.write_audit_log_to_head(
        AuditLogKind::PurgeOpenChangeSets { change_set_ids },
        "purge_change_sets".to_string(),
    )
    .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(PurgeOpenChangeSetsV1Response { success: true }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurgeOpenChangeSetsV1Response {
    #[schema(example = json!({
        "success": "true"
    }))]
    pub success: bool,
}
