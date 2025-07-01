use std::collections::HashSet;

use dal::{
    AttributeValue,
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
    for attribute_value_id in dependent_value_graph.all_value_ids() {
        component_id_set.insert(AttributeValue::component_id(ctx, attribute_value_id).await?);
    }
    let mut component_ids: Vec<_> = component_id_set.into_iter().collect();
    component_ids.sort();

    Ok(DependentValueComponentListMv {
        id: ctx.workspace_pk()?,
        component_ids,
    })
}
