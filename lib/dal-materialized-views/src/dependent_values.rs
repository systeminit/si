use dal::{
    AttributeValue,
    DalContext,
    attribute::value::DependentValueGraph,
    workspace_snapshot::DependentValueRoot,
};
use si_frontend_mv_types::dependent_values::DependentValues as DependentValuesMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.dependent_values",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<DependentValuesMv> {
    let ctx = &ctx;
    let dependent_value_graph = DependentValueGraph::new(
        ctx,
        DependentValueRoot::get_dependent_value_roots(ctx).await?,
    )
    .await?;

    let mut result = DependentValuesMv {
        id: ctx.workspace_pk()?,
        component_attributes: Default::default(),
    };

    for dependent_value in dependent_value_graph.all_value_ids() {
        let (component_id, path) =
            AttributeValue::path_from_component(ctx, dependent_value.attribute_value_id()).await?;
        result
            .component_attributes
            .entry(component_id)
            .or_default()
            .push(path);
    }

    Ok(result)
}
