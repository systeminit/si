use anyhow::Result;
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    diagram::view::{View, ViewView},
    ChangeSet, ChangeSetId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::{view::ViewError, AccessBuilder},
    },
    tracking::track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
}

pub async fn create_view(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(Request { name }): Json<Request>,
) -> Result<ForceChangeSetResponse<ViewView>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    if View::find_by_name(&ctx, name.as_str()).await?.is_some() {
        return Err(ViewError::NameAlreadyInUse(name).into());
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::new(&ctx, name.clone()).await?;
    let view_id = view.clone().id();
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_view",
        serde_json::json!({
            "how": "/diagram/create_view",
            "view_id": view_id,
            "view_name": name.to_owned(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    let view_view = ViewView::from_view(&ctx, view).await?;
    ctx.write_audit_log(AuditLogKind::CreateView { view_id }, name.to_owned())
        .await?;
    WsEvent::view_created(&ctx, view_view.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, view_view))
}
