use dal::change_set::ChangeSet;
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa;

use crate::extract::{PosthogEventTracker, change_set::ChangeSetDalContext};

use crate::service::v1::ChangeSetError;

/// Force apply a change set
#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier")
    ),
    tag = "change_sets",
    responses(
        (status = 200, description = "Change set force applied successfully"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn force_apply(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> Result<(), ChangeSetError> {
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

    Ok(())
}
