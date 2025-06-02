use axum::Json;
use dal::change_set::ChangeSet;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::ChangeSetResult;
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Merge Change Set without approval",
    responses(
        (status = 200, description = "Change Set force applied successfully", body = ForceApplyChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn force_apply(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> ChangeSetResult<Json<ForceApplyChangeSetV1Response>> {
    let change_set_id = ctx.change_set_id();
    let old_status = ctx.change_set()?.status;
    ChangeSet::prepare_for_force_apply(ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        ctx.change_set()?.name.clone(),
    )
    .await?;

    ctx.commit().await?;

    ChangeSet::apply_to_base_change_set(ctx).await?;

    tracker.track(
        ctx,
        "api_apply_change_set",
        json!({
            "merged_change_set": change_set_id,
        }),
    );

    let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name)
        .await?;

    ctx.commit().await?;

    Ok(Json(ForceApplyChangeSetV1Response { success: true }))
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ForceApplyChangeSetV1Response {
    #[schema(example = "true")]
    pub success: bool,
}
