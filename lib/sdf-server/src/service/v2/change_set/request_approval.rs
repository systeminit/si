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

pub async fn request_approval(
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

    change_set.request_change_set_approval(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "request_change_set_approval",
        serde_json::json!({
            "change_set": change_set_id,
        }),
    );
    let change_set_view = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;

    let actor = ctx.history_actor().email(&ctx).await?;
    let change_set_url = format!("https://{host_name}/w/{workspace_pk}/{change_set_id}");
    let message = format!(
        "{} requested an approval of change set {}: {}",
        actor,
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

    ctx.write_audit_log(
        AuditLogKind::RequestChangeSetApproval {
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
