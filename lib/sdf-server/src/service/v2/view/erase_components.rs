use std::collections::HashMap;

use axum::extract::Path;
use axum::{
    Json,
    extract::{Host, OriginalUri},
};
use dal::diagram::geometry::Geometry;
use serde::{Deserialize, Serialize};

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
    track,
};
use dal::diagram::DiagramError;
use dal::diagram::view::{View, ViewComponentsUpdateSingle, ViewId};
use dal::{ChangeSet, ChangeSetId, ComponentId, WorkspacePk, WsEvent};

use super::ViewResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub component_ids: Vec<ComponentId>,
}

pub async fn erase_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(Request { component_ids }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    let mut updated_components: HashMap<_, ViewComponentsUpdateSingle> = HashMap::new();
    // If at least one of the components can be erased, don't blow up if errors happen
    let mut successful_erase = false;
    let mut latest_error = None;
    for component_id in component_ids {
        let geometry = Geometry::get_by_component_and_view(&ctx, component_id, view_id).await?;

        match Geometry::remove(&ctx, geometry.id()).await {
            Ok(_) => {}
            Err(err @ DiagramError::DeletingLastGeometryForComponent(_, _)) => {
                latest_error = Some(err);
                continue;
            }
            Err(err) => return Err(err)?,
        };

        successful_erase = true;

        updated_components
            .entry(view_id)
            .or_default()
            .removed
            .insert(component_id);
    }

    if let Some(err) = latest_error {
        if !successful_erase {
            return Err(err)?;
        }
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

    WsEvent::view_components_update(&ctx, updated_components)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
