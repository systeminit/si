use axum::response::Json;
use dal::change_set::ChangeSet;
use serde::Serialize;
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::ToSchema;

use super::ChangeSetResult;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ChangeSetError,
};

#[utoipa::path(
    delete,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Delete a Change Set",
    responses(
        (status = 200, description = "Change Set deleted successfully", body = DeleteChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn abandon_change_set(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<DeleteChangeSetV1Response>> {
    let maybe_head_changeset = ctx.get_workspace_default_change_set_id().await?;
    if maybe_head_changeset == ctx.change_set_id() {
        return Err(ChangeSetError::CannotAbandonHead);
    }

    let mut change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;
    let old_status = change_set.status;
    ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
        .await?;
    change_set.abandon(ctx).await?;

    tracker.track(
        ctx,
        "api_create_change_set",
        json!({"abandoned_change_set": ctx.change_set_id()}),
    );

    ctx.write_audit_log(
        AuditLogKind::AbandonChangeSet {
            from_status: old_status.into(),
        },
        change_set.name,
    )
    .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(DeleteChangeSetV1Response { success: true }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteChangeSetV1Response {
    #[schema(example = "true")]
    pub success: bool,
}
