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
    let mut component_ids = Component::list_ids(ctx).await?;
    component_ids.sort();

    let workspace_mv_id = ctx.workspace_pk()?;

    Ok(IncomingConnectionsListMv {
        id: workspace_mv_id,
        component_connections: component_ids.iter().copied().map(Into::into).collect(),
    })
}
