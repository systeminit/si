use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use serde::{Deserialize, Serialize};

use dal::{
    diagram::{
        geometry::Geometry,
        view::{View, ViewId},
    },
    ChangeSet, ChangeSetId, WorkspacePk, WsEvent,
};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::ViewResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub view_ids: Vec<ViewId>,
}

pub async fn erase_view_object(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, container_view_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        ViewId,
    )>,
    Json(Request {
        view_ids: component_ids,
    }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::get_by_id(&ctx, container_view_id).await?;

    // let mut updated_components: HashMap<_, ViewComponentsUpdateSingle> = HashMap::new();
    for object_view_id in component_ids {
        let geometry = Geometry::get_by_object_view_and_container_view(
            &ctx,
            object_view_id,
            container_view_id,
        )
        .await?;

        Geometry::remove(&ctx, geometry.id()).await?;

        WsEvent::view_object_erased(&ctx, container_view_id, object_view_id)
            .await?
            .publish_on_commit(&ctx)
            .await?;

        // updated_components
        //     .entry(container_view_id)
        //     .or_default()
        //     .removed
        //     .insert(object_view_id.into());
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "view_object_erased_from_view",
        serde_json::json!({
            "how": "/view/erase_view_object",
            "object_view_id": container_view_id,
            "object_view_name": view.name(),
            "container_view_id": container_view_id,
            "container_view_name": view.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
