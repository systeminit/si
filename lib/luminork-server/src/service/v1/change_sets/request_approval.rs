use axum::{
    Json,
    extract::Host,
};
use dal::{
    Workspace,
    WsEvent,
    change_set::ChangeSet,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ChangeSetError,
    ChangeSetResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
    workspace::WorkspaceAuthorization,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier")
    ),
    tag = "change_sets",
    summary = "Request Change Set merge approval",
    responses(
        (status = 200, description = "Change Set approval requested successfully", body = RequestApprovalChangeSetV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn request_approval(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    WorkspaceAuthorization { workspace_id, .. }: WorkspaceAuthorization,
    tracker: PosthogEventTracker,
    Host(_host_name): Host,
) -> ChangeSetResult<Json<RequestApprovalChangeSetV1Response>> {
    let mut change_set = ctx.change_set()?.clone();
    let change_set_id = change_set.id;
    let base_change_set_id = ctx.get_workspace_default_change_set_id().await?;

    if change_set_id == base_change_set_id {
        return Err(ChangeSetError::CannotMergeHead);
    }

    let old_status = change_set.status;
    let change_set_view = ChangeSet::get_by_id(ctx, change_set_id)
        .await?
        .into_frontend_type(ctx)
        .await?;

    let workspace = Workspace::get_by_pk(ctx, workspace_id).await?;
    if workspace.approvals_enabled() {
        change_set.request_change_set_approval(ctx).await?;
        tracker.track(
            ctx,
            "api_request_change_set_approval",
            json!({
                "change_set_name": change_set.name,
            }),
        );

        ctx.write_audit_log(
            AuditLogKind::RequestChangeSetApproval {
                from_status: old_status.into(),
            },
            change_set_view.name.clone(),
        )
        .await?;

        WsEvent::change_set_status_changed(ctx, old_status, change_set_view)
            .await?
            .publish_on_commit(ctx)
            .await?;
    } else {
        ChangeSet::force_change_set_approval(ctx).await?;
        ctx.write_audit_log(
            AuditLogKind::ApproveChangeSetApply {
                from_status: old_status.into(),
            },
            ctx.change_set()?.name.clone(),
        )
        .await?;

        // We need to run a commit before apply so changes get saved
        ctx.commit_no_rebase().await?;

        ChangeSet::begin_apply(ctx).await?;

        tracker.track(
            ctx,
            "api_apply_change_set",
            json!({
                "merged_change_set": change_set_id,
            }),
        );

        let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;
        ctx.write_audit_log(AuditLogKind::ApplyChangeSet, change_set.name)
            .await?;
    }

    ctx.commit().await?;

    Ok(Json(RequestApprovalChangeSetV1Response { success: true }))
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestApprovalChangeSetV1Response {
    #[schema(example = "true")]
    pub success: bool,
}
