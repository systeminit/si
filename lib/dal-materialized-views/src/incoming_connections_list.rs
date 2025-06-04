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
    let mut component_connections = Vec::with_capacity(component_ids.len());

    for component_id in component_ids {
        component_connections
            .push(super::incoming_connections::assemble(ctx.clone(), component_id).await?);
    }
    let workspace_mv_id = ctx.workspace_pk()?;

    Ok(IncomingConnectionsListMv {
        id: workspace_mv_id,
        component_connections: component_connections.iter().map(Into::into).collect(),
    })
}
