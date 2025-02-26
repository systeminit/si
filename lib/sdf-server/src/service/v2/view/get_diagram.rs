use std::collections::HashMap;

use anyhow::Result;
use axum::extract::{Json, Path};
use dal::{
    diagram::{
        geometry::{Geometry, GeometryRepresents},
        view::{View, ViewId, ViewView},
        Diagram, DiagramError,
    },
    slow_rt, ChangeSetId, ComponentId, DalContext, WorkspacePk,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::RawGeometry;
use telemetry::prelude::debug;

use crate::{extract::HandlerContext, service::v2::AccessBuilder};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeometryResponse {
    view_id: ViewId,
    name: String,
    components: HashMap<ComponentId, RawGeometry>,
    views: HashMap<ViewId, RawGeometry>,
}

pub async fn get_geometry(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
) -> Result<Json<GeometryResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    let mut components = HashMap::new();
    let mut views = HashMap::new();

    for geometry in Geometry::list_by_view_id(&ctx, view_id).await? {
        let geo_represents = match Geometry::represented_id(&ctx, geometry.id()).await {
            Ok(id) => id,
            Err(error) => match error.downcast_ref::<DiagramError>() {
                Some(DiagramError::RepresentedNotFoundForGeometry(geo_id)) => {
                    let changeset_id = ctx.change_set_id();
                    // NOTE(victor): The first version of views didn't delete geometries with components,
                    // so we have dangling geometries in some workspaces. We should clean this up at some point,
                    // but we just skip orphan geometries here to make assemble work.

                    debug!(
                        si.change_set.id = %changeset_id,
                        si.geometry.id = %geo_id,
                        "Could not find component for geometry - skipping"
                    );

                    continue;
                }
                _ => return Err(error),
            },
        };

        match geo_represents {
            GeometryRepresents::Component(component_id) => {
                components.insert(component_id, geometry.into_raw());
            }
            GeometryRepresents::View(view_id) => {
                views.insert(view_id, geometry.into_raw());
            }
        }
    }

    Ok(Json(GeometryResponse {
        view_id,
        name: view.name().to_string(),
        components,
        views,
    }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    view: ViewView,
    diagram: Diagram,
}

pub async fn get_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
) -> Result<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    get_diagram_inner(&ctx, view).await
}

pub async fn get_default_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let view_id = View::get_id_for_default(&ctx).await?;
    let view = View::get_by_id(&ctx, view_id).await?;

    get_diagram_inner(&ctx, view).await
}

async fn get_diagram_inner(ctx: &DalContext, view: View) -> Result<Json<Response>> {
    let ctx_clone = ctx.clone();
    let view_id = view.id();
    let diagram = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Ok::<Diagram, anyhow::Error>(Diagram::assemble(ctx, Some(view_id)).await?)
    })?
    .await??;

    Ok(Json(Response {
        view: ViewView::from_view(ctx, view).await?,
        diagram,
    }))
}
