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
    let component_ids = Component::list_ids(ctx).await?;
    let mut components = Vec::with_capacity(component_ids.len());

    for component_id in component_ids {
        components.push(super::component::assemble(ctx.clone(), component_id).await?);
    }

    Ok(ComponentListMv {
        id: ctx.change_set_id(),
        components: components.iter().map(Into::into).collect(),
    })
}
