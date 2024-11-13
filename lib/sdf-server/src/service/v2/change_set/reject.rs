use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WsEvent};
use si_events::audit_log::AuditLogKind;

use super::{Error, Result};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn reject(
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

    let mut change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?;
    let old_status = change_set.status;

    change_set.reject_change_set_for_apply(&ctx).await?;

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

    let change_set_view = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?
        .into_frontend_type(&ctx)
        .await?;
    ctx.write_audit_log(
        AuditLogKind::RejectChangeSetApply {
            from_status: old_status.into(),
        },
        change_set_view.name.clone(),
    )
    .await?;
    WsEvent::change_set_status_changed(&ctx, old_status, change_set_view)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
