use axum::{
    extract::{Host, OriginalUri, Path, State},
    Json,
};
use dal::{change_set::approval::ChangeSetApproval, ChangeSet, ChangeSetId, WorkspacePk, WsEvent};
use serde::Deserialize;
use si_events::{audit_log::AuditLogKind, ChangeSetApprovalStatus};

use super::{ChangeSetAPIError, Error, Result};
use crate::{
    dal_wrapper,
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track, AppState,
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
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    State(mut state): State<AppState>,
    Json(request): Json<Request>,
) -> Result<()> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
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

    // Cache the old status.
    let change_set = ChangeSet::get_by_id(&ctx, ctx.visibility().change_set_id).await?;
    let old_status = change_set.status;

    // This is the core of the route!
    let spicedb_client = state
        .spicedb_client()
        .ok_or(ChangeSetAPIError::SpiceDBClientNotFound)?;
    let approving_ids_with_hashes =
        dal_wrapper::change_set_approval::determine_approving_ids_with_hashes(&ctx, spicedb_client)
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

    ctx.commit().await?;

    Ok(())
}
