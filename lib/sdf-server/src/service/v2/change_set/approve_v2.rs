use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{change_set::approval::ChangeSetApproval, ChangeSet, ChangeSetId, WorkspacePk};
use serde::Deserialize;
use si_events::{audit_log::AuditLogKind, ChangeSetApprovalStatus};

use super::{Error, Result};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub status: ChangeSetApprovalStatus,
}

pub async fn approve(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<Request>,
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

    let change_set = ChangeSet::find(&ctx, ctx.visibility().change_set_id)
        .await?
        .ok_or(Error::ChangeSetNotFound(ctx.change_set_id()))?;
    ChangeSetApproval::new(&ctx, request.status).await?;

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
            from_status: change_set.status.into(),
        },
        change_set.name,
    )
    .await?;

    ctx.commit().await?;

    Ok(())
}
