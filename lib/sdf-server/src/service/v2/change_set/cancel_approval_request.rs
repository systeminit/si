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

use super::{
    Result,
    post_to_webhook,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
    track,
};

pub async fn cancel_approval_request(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let mut change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    let old_status = change_set.status;

    change_set.reopen_change_set(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "approve_change_set_apply",
        serde_json::json!({
            "merged_change_set": change_set_id,
        }),
    );

    let change_set_view = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    ctx.write_audit_log(
        AuditLogKind::WithdrawRequestForChangeSetApply {
            from_status: old_status.into(),
        },
        change_set_view.name.clone(),
    )
    .await?;
    WsEvent::change_set_status_changed(&ctx, old_status, change_set_view.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;
    let actor = ctx.history_actor().email(&ctx).await?;
    let change_set_url = format!("https://{host_name}/w/{workspace_pk}/{change_set_id}");
    let message = format!(
        "{} withdrew approval request of change set {}: {}",
        actor,
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

    ctx.commit().await?;

    Ok(())
}
