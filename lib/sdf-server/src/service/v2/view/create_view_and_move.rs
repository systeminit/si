use std::collections::HashMap;

use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::force_change_set_response::ForceChangeSetResponse;
use crate::service::v2::view::{ViewError, ViewResult};
use crate::tracking::track;
use axum::extract::{Host, OriginalUri, Path};
use axum::Json;
use dal::diagram::geometry::Geometry;
use dal::diagram::view::{View, ViewComponentsUpdateList, ViewId, ViewView};
use dal::{ChangeSet, ChangeSetId, Component, ComponentError, ComponentId, WorkspacePk, WsEvent};
use serde::{Deserialize, Serialize};
use si_frontend_types::{RawGeometry, StringGeometry};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ViewNodeGeometry {
    pub x: String,
    pub y: String,
    pub radius: String,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
    pub source_view_id: ViewId,
    pub geometries_by_component_id: HashMap<ComponentId, StringGeometry>,
    pub remove_from_original_view: bool,
    pub place_view_at: ViewNodeGeometry,
}

pub async fn create_view_and_move(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(Request {
        name,
        source_view_id,
        geometries_by_component_id,
        remove_from_original_view,
        place_view_at,
    }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<ViewView>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    if View::find_by_name(&ctx, name.as_str()).await?.is_some() {
        return Err(ViewError::NameAlreadyInUse(name));
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::new(&ctx, name.clone()).await?;
    let view_id = view.id();

    let view_view = ViewView::from_view(&ctx, view).await?;

    WsEvent::view_created(&ctx, view_view.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let mut updated_components = ViewComponentsUpdateList::new();

    let mut successful_erase = false;
    let mut latest_error = None;
    for (component_id, string_geometry) in geometries_by_component_id.clone() {
        let geometry: RawGeometry = string_geometry.try_into()?;

        match Component::add_to_view(&ctx, component_id, view_id, geometry.clone()).await {
            Ok(_) => {}
            Err(err @ ComponentError::ComponentAlreadyInView(_, _)) => {
                latest_error = Some(err);
                continue;
            }
            Err(err) => return Err(err)?,
        };

        successful_erase = true;

        updated_components
            .entry(view_id)
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

    if let Some(err) = latest_error {
        if !successful_erase {
            return Err(err)?;
        }
    }

    WsEvent::view_components_update(&ctx, updated_components)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let (Ok(x), Ok(y), Ok(radius)) = (
        place_view_at.x.clone().parse::<isize>(),
        place_view_at.y.clone().parse::<isize>(),
        place_view_at.radius.clone().parse::<isize>(),
    ) else {
        ctx.rollback().await?;
        return Err(ViewError::InvalidRequest(
            "geometry unable to be parsed from create view object request".into(),
        ));
    };

    let geometry = RawGeometry {
        x,
        y,
        width: Some(radius),
        height: Some(radius),
    };
    View::add_to_another_view(&ctx, view_id, source_view_id, geometry.clone()).await?;

    WsEvent::view_object_created(&ctx, source_view_id, view_id, geometry)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_view",
        serde_json::json!({
            "how": "/diagram/create_view_and_move",
            "view_id": view_id,
            "view_name": name,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, view_view))
}
