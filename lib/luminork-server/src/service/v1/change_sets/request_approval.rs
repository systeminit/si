use axum::extract::Host;
use dal::{WsEvent, change_set::ChangeSet};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa;

use crate::extract::{
    PosthogEventTracker, change_set::ChangeSetDalContext, workspace::WorkspaceAuthorization,
};

use crate::service::v1::ChangeSetError;

/// Request approval for a change set
#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier")
    ),
    tag = "change_sets",
    responses(
        (status = 200, description = "Change set approval requested successfully"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn request_approval(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    WorkspaceAuthorization { .. }: WorkspaceAuthorization,
    tracker: PosthogEventTracker,
    Host(_host_name): Host,
) -> Result<(), ChangeSetError> {
    let mut change_set = ctx.change_set()?.clone();
    let change_set_id = change_set.id;
    let old_status = change_set.status;

    change_set.request_change_set_approval(ctx).await?;

    tracker.track(
        ctx,
        "api_request_change_set_approval",
        json!({
            "change_set": change_set.id,
        }),
    );

    let change_set_view = ChangeSet::get_by_id(ctx, change_set_id)
        .await?
        .into_frontend_type(ctx)
        .await?;

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

    ctx.commit().await?;

    Ok(())
}
