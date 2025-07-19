use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
        State,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
    WsEvent,
    change_set::approval::ChangeSetApproval,
    workspace_snapshot::DependentValueRoot,
};
use sdf_core::dal_wrapper;
use serde::Deserialize;
use si_events::{
    ChangeSetApprovalStatus,
    audit_log::AuditLogKind,
};

use super::{
    ChangeSetAPIError,
    Error,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub status: ChangeSetApprovalStatus,
}

#[allow(clippy::too_many_arguments)]
pub async fn approve(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    State(mut state): State<AppState>,
    Json(request): Json<Request>,
) -> Result<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Ensure that DVU roots are empty before continuing?
    // todo(brit): maybe we can get away without this. Ex: Approve a PR before tests finish
    if DependentValueRoot::roots_exist(&ctx).await? {
        // TODO(nick): we should consider requiring this check in integration tests too. Why did I
        // not do this at the time of writing? Tests have multiple ways to call "apply", whether
        // its via helpers or through the change set methods directly. In addition, they test
        // for success and failure, not solely for success. We should still do this, but not within
        // the PR corresponding to when this message was written.
        return Err(Error::DvuRootsNotEmpty(ctx.change_set_id()));
    }

    // Cache the old status.
    let change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    let old_status = change_set.status;

    // This is the core of the route!
    let spicedb_client = state
        .spicedb_client()
        .ok_or(ChangeSetAPIError::SpiceDBClientNotFound)?;
    let approving_ids_with_hashes =
        dal_wrapper::change_set::new_approval_approving_ids_with_hashes(&ctx, spicedb_client)
            .await?;
    ChangeSetApproval::new(&ctx, request.status, approving_ids_with_hashes).await?;

    WsEvent::change_set_approval_status_changed(&ctx, ctx.change_set_id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    // Tracking, audit logging, etc.
    {
        match request.status {
            // NOTE(nick): this matches what was in the original approve route.
            ChangeSetApprovalStatus::Approved => {
                track(
                    &posthog_client,
                    &ctx,
                    &original_uri,
                    &host_name,
                    "approve_change_set_apply",
                    serde_json::json!({
                        "merged_change_set": change_set.id,
                    }),
                );
                ctx.write_audit_log(
                    AuditLogKind::ApproveChangeSetApply {
                        from_status: old_status.into(),
                    },
                    change_set.name,
                )
                .await?;
            }
            ChangeSetApprovalStatus::Rejected => {
                // NOTE(nick): this matches what was in the original reject route.
                track(
                    &posthog_client,
                    &ctx,
                    &original_uri,
                    &host_name,
                    "reject_change_set_apply",
                    serde_json::json!({
                        "change_set": change_set.id,
                    }),
                );
                ctx.write_audit_log(
                    AuditLogKind::RejectChangeSetApply {
                        from_status: old_status.into(),
                    },
                    change_set.name,
                )
                .await?;
            }
        }
    }

    let change_set_view = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    let actor = ctx.history_actor().email(&ctx).await?;
    let change_set_url = format!("https://{host_name}/w/{workspace_pk}/{change_set_id}");
    let message = format!(
        "{} {} merge of change set {}: {}",
        actor,
        request.status,
        change_set_view.name.clone(),
        change_set_url
    );
    post_to_webhook(&ctx, workspace_pk, message.as_str()).await?;

    ctx.commit().await?;

    Ok(())
}
