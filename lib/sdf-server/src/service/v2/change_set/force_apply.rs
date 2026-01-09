use axum::extract::{
    Host,
    OriginalUri,
    Path,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use si_events::audit_log::AuditLogKind;

use super::Result;
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
    track,
};

pub async fn force_apply(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;
    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;
    let old_status = change_set.status;
    ChangeSet::force_change_set_approval(&ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        change_set.name,
    )
    .await?;
    // We need to run a commit before apply so changes get saved
    ctx.commit().await?;

    ChangeSet::begin_apply(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": change_set_id,
        }),
    );

    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name)
        .await?;
    // Ws Event fires from the dal

    ctx.commit().await?;

    Ok(())
}
