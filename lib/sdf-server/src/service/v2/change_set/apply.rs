use axum::extract::{
    Host,
    OriginalUri,
    Path,
    State,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use sdf_core::dal_wrapper;
use si_events::audit_log::AuditLogKind;

use super::{
    ChangeSetAPIError,
    Result,
    post_to_webhook,
};
use crate::{
    AppState,
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
    track,
};

pub async fn apply(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    State(mut state): State<AppState>,
) -> Result<()> {
    let mut ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;
    let spicedb_client = state
        .spicedb_client()
        .ok_or(ChangeSetAPIError::SpiceDBClientNotFound)?;

    // Perform the protected apply flow.
    dal_wrapper::change_set::protected_apply_to_base_change_set(&mut ctx, spicedb_client).await?;

    // Tracking, audit logging, etc.
    {
        let change_set_view = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id)
            .await?
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

        ctx.write_audit_log(
            AuditLogKind::ApplyChangeSet,
            change_set_view.name.to_owned(),
        )
        .await?;

        let actor = ctx.history_actor().email(&ctx).await?;
        let change_set_url = format!("https://{host_name}/w/{workspace_pk}/{change_set_id}");
        let message = format!(
            "{} applied change set {} to HEAD: {}",
            actor, change_set_view.name, change_set_url
        );
        post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

        // WS Event fires from the dal
        ctx.commit().await?;
    }

    Ok(())
}
