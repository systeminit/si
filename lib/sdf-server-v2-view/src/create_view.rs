use crate::{ViewError, ViewResult, ViewView};
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use axum_util::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use dal::{diagram::view::View, ChangeSet, ChangeSetId, WorkspacePk};
use serde::{Deserialize, Serialize};

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
) -> ViewResult<ForceChangeSetResponse<ViewView>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    if View::find_by_name(&ctx, name.as_str()).await?.is_some() {
        return Err(ViewError::NameAlreadyInUse(name));
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::new(&ctx, name).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_view",
        serde_json::json!({
            "how": "/diagram/create_view",
            "view_id": view.id(),
            "view_name": view.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        ViewView::from_view(&ctx, view).await?,
    ))
}
