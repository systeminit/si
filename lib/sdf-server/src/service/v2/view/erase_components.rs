use std::collections::HashMap;

use axum::extract::Path;
use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::diagram::geometry::Geometry;
use serde::{Deserialize, Serialize};

use dal::diagram::view::{View, ViewComponentsUpdateSingle, ViewId};
use dal::{ChangeSet, ChangeSetId, ComponentId, WorkspacePk, WsEvent};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::ViewResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub component_ids: Vec<ComponentId>,
    pub client_ulid: ulid::Ulid,
}

pub async fn erase_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(Request {
        component_ids,
        client_ulid,
    }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    let mut updated_components: HashMap<_, ViewComponentsUpdateSingle> = HashMap::new();
    for component_id in component_ids {
        let geometry = Geometry::get_by_component_and_view(&ctx, component_id, view_id).await?;

        Geometry::remove(&ctx, geometry.id()).await?;

        updated_components
            .entry(view_id)
            .or_default()
            .removed
            .insert(component_id.into());
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "component_erased_from_view",
        serde_json::json!({
            "how": "/view/erase_components",
            "view_id": view_id,
            "view_name": view.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    WsEvent::view_components_update(&ctx, updated_components, Some(client_ulid))
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
