use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    ComponentError,
    ComponentId,
    Visibility,
    WsEvent,
    diagram::{
        geometry::Geometry,
        view::{
            View,
            ViewComponentsUpdateList,
            ViewId,
        },
    },
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::{
    RawGeometry,
    StringGeometry,
};

use super::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub source_view_id: ViewId,
    pub destination_view_id: ViewId,
    pub geometries_by_component_id: HashMap<ComponentId, StringGeometry>,
    pub remove_from_original_view: bool,
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
    }): Json<Request>,
) -> DiagramResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(visibility.change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let destination_view = View::get_by_id(&ctx, destination_view_id).await?;

    let mut updated_components = ViewComponentsUpdateList::new();

    let mut successful_erase = false;
    let mut latest_error = None;
    for (component_id, string_geometry) in geometries_by_component_id.clone() {
        let geometry: RawGeometry = string_geometry.try_into()?;

        match Component::add_to_view(&ctx, component_id, destination_view_id, geometry.clone())
            .await
        {
            Ok(_) => {}
            Err(err @ ComponentError::ComponentAlreadyInView(_, _)) => {
                latest_error = Some(err);
                continue;
            }
            Err(err) => return Err(err)?,
        };

        successful_erase = true;

        updated_components
            .entry(destination_view_id)
            .or_default()
            .added
            .insert(component_id, geometry);

        if remove_from_original_view {
            let old_geometry =
                Geometry::get_by_component_and_view(&ctx, component_id, source_view_id).await?;

            updated_components
                .entry(source_view_id)
                .or_default()
                .removed
                .insert(component_id);

            Geometry::remove(&ctx, old_geometry.id()).await?
        }
    }

    if let Some(err) = latest_error {
        if !successful_erase {
            return Err(err)?;
        }
    }

    WsEvent::view_components_update(&ctx, updated_components)
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
