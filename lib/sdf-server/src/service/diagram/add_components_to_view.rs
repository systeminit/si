use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::diagram::DiagramResult;
use crate::service::force_change_set_response::ForceChangeSetResponse;
use crate::tracking::track;
use axum::extract::{Host, OriginalUri};
use axum::Json;
use dal::diagram::geometry::Geometry;
use dal::diagram::view::{View, ViewComponentsUpdateList, ViewId};
use dal::{ChangeSet, Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use si_frontend_types::{RawGeometry, StringGeometry};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub source_view_id: ViewId,
    pub destination_view_id: ViewId,
    pub geometries_by_component_id: HashMap<ComponentId, StringGeometry>,
    pub remove_from_original_view: bool,
    pub client_ulid: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn add_components_to_view(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(Request {
        source_view_id,
        destination_view_id,
        geometries_by_component_id,
        remove_from_original_view,
        visibility,
        client_ulid,
    }): Json<Request>,
) -> DiagramResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(visibility.change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let destination_view = View::get_by_id(&ctx, destination_view_id).await?;

    let mut updated_components = ViewComponentsUpdateList::new();

    for (component_id, string_geometry) in geometries_by_component_id.clone() {
        let geometry: RawGeometry = string_geometry.try_into()?;

        Component::add_to_view(&ctx, component_id, destination_view_id, geometry.clone()).await?;
        updated_components
            .entry(destination_view_id)
            .or_default()
            .added
            .insert(component_id.into(), geometry);

        if remove_from_original_view {
            let old_geometry =
                Geometry::get_by_component_and_view(&ctx, component_id, source_view_id).await?;

            updated_components
                .entry(source_view_id)
                .or_default()
                .removed
                .insert(component_id.into());

            Geometry::remove(&ctx, old_geometry.id()).await?
        }
    }

    WsEvent::view_components_update(&ctx, updated_components, Some(client_ulid))
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "add_components_to_view",
        serde_json::json!({
            "how": "/diagram/add_components_to_view",
            "destination_view_id": destination_view.id(),
            "destination_view_name": destination_view.name(),
            "remove_from_original_view": remove_from_original_view,
            "component_count": geometries_by_component_id.len(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
