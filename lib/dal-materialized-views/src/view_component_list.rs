use dal::{
    DalContext,
    diagram::geometry::{
        Geometry,
        GeometryRepresents,
    },
};
use si_frontend_mv_types::view::ViewComponentList as ViewComponentListMv;
use si_id::ViewId;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.view_component_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, view_id: ViewId) -> super::Result<ViewComponentListMv> {
    // Logic comes from the /get_geometry endpoint
    let mut components = vec![];

    for geometry in Geometry::list_by_view_id(&ctx, view_id).await? {
        let geo_represents = match Geometry::represented_id(&ctx, geometry.id()).await? {
            Some(id) => id,
            None => continue,
        };

        match geo_represents {
            GeometryRepresents::Component(component_id) => {
                components.push(component_id);
            }
            GeometryRepresents::View(_view_id) => {}
        }
    }
    components.sort();

    Ok(ViewComponentListMv {
        id: view_id,
        components: components.iter().map(|&id| id.into()).collect(),
    })
}
