use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WsEvent};
use si_events::audit_log::AuditLogKind;

use super::{post_to_webhook, Error, Result};
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track,
};

pub async fn approve(
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

    // Ensure that DVU roots are empty before continuing?
    // todo(brit): maybe we can get away without this. Ex: Approve a PR before tests finish
    if !ctx
        .workspace_snapshot()?
        .get_dependent_value_roots()
        .await?
        .is_empty()
    {
        // TODO(nick): we should consider requiring this check in integration tests too. Why did I
        // not do this at the time of writing? Tests have multiple ways to call "apply", whether
        // its via helpers or through the change set methods directly. In addition, they test
        // for success and failure, not solely for success. We should still do this, but not within
        // the PR corresponding to when this message was written.
        return Err(Error::DvuRootsNotEmpty(ctx.change_set_id()));
    }

    let mut change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    let old_status = change_set.status;
    change_set.approve_change_set_for_apply(&ctx).await?;

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
        AuditLogKind::ApproveChangeSetApply {
            from_status: old_status.into(),
        },
        change_set_view.name.clone(),
    )
    .await?;

    let actor = ctx.history_actor().email(&ctx).await?;
    let change_set_url = format!("https://{}/w/{}/{}", host_name, workspace_pk, change_set_id);
    let message = format!(
        "{} approved merge of change set {}: {}",
        actor,
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

    WsEvent::change_set_status_changed(&ctx, old_status, change_set_view)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
