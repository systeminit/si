use crate::extract::{HandlerContext, PosthogClient};
use axum::Json;
use axum::extract::{Host, OriginalUri, Path};
use dal::component::frame::Frame;
use dal::diagram::geometry::Geometry;
use serde::{Deserialize, Serialize};
use si_id::ViewId;
use std::collections::HashMap;

use dal::diagram::view::{View, ViewComponentsUpdateList, ViewView};
use dal::{
    ChangeSet, ChangeSetId, Component, ComponentError, ComponentId, ComponentType, WorkspacePk,
    WsEvent,
};

use crate::service::force_change_set_response::ForceChangeSetResponse;
use crate::service::v2::AccessBuilder;
use crate::service::v2::view::{ViewError, ViewNodeGeometry, ViewResult};
use si_frontend_types::{RawGeometry, StringGeometry};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub component_id: ComponentId,
    pub contained_component_ids: HashMap<ComponentId, StringGeometry>,
    pub source_view_id: ViewId,
    pub place_view_at: ViewNodeGeometry,
}

pub async fn convert_to_view(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(Request {
        component_id,
        mut contained_component_ids,
        source_view_id,
        place_view_at,
    }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<ViewView>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    if component.get_type(&ctx).await? == ComponentType::Component {
        return Err(ViewError::ComponentIsNotAFrame(component_id));
    }

    let variant = component.schema_variant(&ctx).await?;
    let component_name = component.name(&ctx).await?;

    if variant.display_name() == "Generic Frame" {
        let maybe_eventual_parent = component.parent(&ctx).await?;
        let child_components_ids = Component::get_children_for_id(&ctx, component_id).await?;
        for child_comp_id in child_components_ids {
            let child = Component::get_by_id(&ctx, child_comp_id).await?;
            if let Some(parent_component) = child.parent(&ctx).await? {
                if parent_component == component.id() {
                    match maybe_eventual_parent {
                        Some(eventual_parent) => {
                            Frame::upsert_parent_no_events(&ctx, child_comp_id, eventual_parent)
                                .await?;
                        }
                        None => {
                            Frame::orphan_child(&ctx, child_comp_id).await?;
                        }
                    }
                }
            }
        }

        component.delete(&ctx).await?;
        WsEvent::component_deleted(&ctx, component_id)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    } else {
        contained_component_ids.insert(
            component_id,
            StringGeometry {
                x: place_view_at.x.clone(),
                y: place_view_at.y.clone(),
                height: None,
                width: None,
            },
        );
    };

    let view = View::new(&ctx, component_name.clone()).await?;
    let view_id = view.id();
    let view_view = ViewView::from_view(&ctx, view).await?;
    WsEvent::view_created(&ctx, view_view.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let mut updated_components = ViewComponentsUpdateList::new();
    let mut successful_erase = false;
    let mut latest_error = None;

    // Move the components to the new view
    for (component_id, string_geometry) in contained_component_ids {
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
            .insert(component_id, geometry);

        let old_geometry =
            Geometry::get_by_component_and_view(&ctx, component_id, source_view_id).await?;

        updated_components
            .entry(source_view_id)
            .or_default()
            .removed
            .insert(component_id);

        Geometry::remove(&ctx, old_geometry.id()).await?
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

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, view_view))
}
