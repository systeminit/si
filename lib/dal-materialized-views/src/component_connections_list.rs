use dal::{
    Component,
    DalContext,
};
use si_frontend_types::newhotness::component_connections::ComponentConnectionsListBeta as ComponentConnectionsListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.component_connections_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<ComponentConnectionsListMv> {
    let ctx = &ctx;
    let component_ids = Component::list_ids(ctx).await?;
    let mut component_connections = Vec::with_capacity(component_ids.len());

    for component_id in component_ids {
        component_connections
            .push(super::component_connections::assemble(ctx.clone(), component_id).await?);
    }

    Ok(ComponentConnectionsListMv {
        id: ctx.change_set_id(),
        component_connections: component_connections.iter().map(Into::into).collect(),
    })
}
