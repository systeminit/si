use axum::extract::{
    Host,
    OriginalUri,
    Path,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
    WsEvent,
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

pub async fn reopen(
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

    let mut change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    let old_status = change_set.status;

    //todo(brit): should we guard against re-opening abandoned change sets?
    // this might be helpful if we don't...
    change_set.reopen_change_set(&ctx).await?;

    let change_set_view = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;

    ctx.write_audit_log(
        AuditLogKind::ReopenChangeSet {
            from_status: old_status.into(),
        },
        change_set_view.name.clone(),
    )
    .await?;
    WsEvent::change_set_status_changed(&ctx, old_status, change_set_view)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "reject_change_set_apply",
        serde_json::json!({
            "change_set": change_set_id,
        }),
    );

    ctx.commit().await?;

    Ok(())
}
