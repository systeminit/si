use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use si_events::audit_log::AuditLogKind;

use super::{post_to_webhook, Error, Result};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn apply(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<()> {
    let mut ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;
    let change_set = ChangeSet::find(&ctx, change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?;
    ChangeSet::prepare_for_apply(&ctx).await?;

    // We need to run a commit before apply so changes get saved
    ctx.commit().await?;

    ChangeSet::apply_to_base_change_set(&mut ctx).await?;

    let change_set_view = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?
        .into_frontend_type(&ctx)
        .await?;

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

    ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name)
        .await?;

    let actor = ctx.history_actor().email(&ctx).await?;
    let change_set_url = format!("https://{}/w/{}/{}", host_name, workspace_pk, change_set_id);
    let message = format!(
        "{} applied change set {} to HEAD: {}",
        actor,
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

    // WS Event fires from the dal
    ctx.commit().await?;

    Ok(())
}
