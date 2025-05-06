use dal::{
    DalContext,
    diagram::{
        DiagramError,
        geometry::{
            Geometry,
            GeometryRepresents,
        },
    },
};
use si_frontend_types::view::ViewComponentList;
use si_id::ViewId;

use crate::component::as_frontend_type;

/// Generates a [`ViewComponentList`] MV.
pub async fn components_as_frontend_list_type(
    ctx: DalContext,
    view_id: ViewId,
) -> super::Result<ViewComponentList> {
    // Logic comes from the /get_geometry endpoint
    let mut components = vec![];

    for geometry in Geometry::list_by_view_id(&ctx, view_id).await? {
        let geo_represents = match Geometry::represented_id(&ctx, geometry.id()).await {
            Ok(id) => id,
            Err(DiagramError::RepresentedNotFoundForGeometry(_geo_id)) => {
                // NOTE(victor): The first version of views didn't delete geometries with components,
                // so we have dangling geometries in some workspaces. We should clean this up at some point,
                // but we just skip orphan geometries here to make assemble work.

                continue;
            }
            Err(err) => return Err(err)?,
        };

        match geo_represents {
            GeometryRepresents::Component(component_id) => {
                components.push(as_frontend_type(ctx.clone(), component_id).await?);
            }
            GeometryRepresents::View(_view_id) => {}
        }
    }

    Ok(ViewComponentList {
        id: view_id,
        components: components.iter().map(Into::into).collect(),
    })
}
