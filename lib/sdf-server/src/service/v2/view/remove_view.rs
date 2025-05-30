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
    diagram::view::{
        View,
        ViewId,
    },
};
use si_events::audit_log::AuditLogKind;

use super::{
    ViewError,
    ViewResult,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
};

pub async fn remove_view(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let current_view_list = View::list(&ctx).await?;
    if current_view_list.len() == 1 {
        return Err(ViewError::CantDeleteOnlyView());
    }

    let view = View::get_by_id(&ctx, view_id).await?;
    View::remove(&ctx, view_id).await?;

    WsEvent::view_deleted(&ctx, view_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.write_audit_log(
        AuditLogKind::DeleteView { view_id: view.id() },
        view.name().to_owned(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "remove_view",
        serde_json::json!({
            "how": "/view",
            "view_id": view.id(),
            "view_name": view.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
