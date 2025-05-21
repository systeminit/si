use dal::{
    Component,
    DalContext,
};
use si_frontend_mv_types::incoming_connections::IncomingConnectionsList as IncomingConnectionsListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.incoming_connections_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<IncomingConnectionsListMv> {
    let ctx = &ctx;
    let component_ids = Component::list_ids(ctx).await?;
    // let mut component_connections = Vec::with_capacity(component_ids.len());

    // for component_id in component_ids {
    //     component_connections
    //         .push(super::incoming_connections::assemble(ctx.clone(), component_id).await?);
    // }

    Ok(IncomingConnectionsListMv {
        id: ctx.change_set_id(),
        // if this is a reference we'll build every incoming_connection twice: once here, regardless of if the component
        // has actually changed, and once in the individual MV creation when the component merklehash changes
        component_connections: component_ids.iter().map(|&id| id.into()).collect(),
    })
}
