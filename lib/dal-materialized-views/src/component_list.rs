use dal::{
    Component,
    DalContext,
};
use si_frontend_mv_types::component::ComponentList as ComponentListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.component_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<ComponentListMv> {
    let ctx = &ctx;
    let mut component_ids = Component::list_ids(ctx).await?;
    component_ids.sort();
   

    Ok(ComponentListMv {
        id: ctx.change_set_id(),
        components: component_ids.iter().map(|&id| id.into()).collect(),
    })
}
