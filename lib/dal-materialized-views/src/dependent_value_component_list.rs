use std::collections::HashSet;

use dal::{
    DalContext,
    attribute::value::DependentValueGraph,
    workspace_snapshot::DependentValueRoot,
};
use si_frontend_mv_types::dependent_values::DependentValueComponentList as DependentValueComponentListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.dependent_value_component_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<DependentValueComponentListMv> {
    let ctx = &ctx;
    let dependent_value_graph = DependentValueGraph::new(
        ctx,
        DependentValueRoot::get_dependent_value_roots(ctx).await?,
    )
    .await?;

    let mut component_id_set = HashSet::new();
    for dependent_value in dependent_value_graph.all_value_ids() {
        if let Some(component_id) =
            dependent_value_graph.cached_component_id_for_value(dependent_value)
        {
            component_id_set.insert(component_id);
        }
    }
    let mut component_ids: Vec<_> = component_id_set.into_iter().collect();
    component_ids.sort();

    Ok(DependentValueComponentListMv {
        id: ctx.workspace_pk()?,
        component_ids,
    })
}
