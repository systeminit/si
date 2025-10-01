use axum::extract::Path;
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use si_events::audit_log::AuditLogKind;

use super::{
    ChangeSetAPIError,
    Result,
};

pub async fn abandon(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    if ctx.is_head().await? {
        return Err(ChangeSetAPIError::CannotAbandonHead);
    }

    let mut change_set = ChangeSet::get_by_id(ctx, change_set_id).await?;
    let old_status = change_set.status;
    // Skipping the load of the snapshot here as it is not required and be expensive
    ctx.update_visibility_deprecated(change_set.id.into());
    change_set.abandon(ctx).await?;

    ctx.write_audit_log(
        AuditLogKind::AbandonChangeSet {
            from_status: old_status.into(),
        },
        change_set.name,
    )
    .await?;

    tracker.track(
        ctx,
        "abandon_change_set",
        serde_json::json!({
            "abandoned_change_set": change_set_id,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(())
}
