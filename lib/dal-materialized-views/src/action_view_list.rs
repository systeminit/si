use dal::{
    Component,
    DalContext,
    Func,
    action::{
        Action,
        dependency_graph::ActionDependencyGraph,
        prototype::ActionPrototype,
    },
};
use si_frontend_mv_types::action::{
    ActionView,
    ActionViewList as ActionViewListMv,
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.action_view_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<ActionViewListMv> {
    let ctx = &ctx;
    let action_ids = Action::list_topologically(ctx).await?;

    let mut views = Vec::new();

    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
    if !action_graph.is_acyclic() {
        warn!("action graph has a cycle");
    }

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;

        let prototype_id = Action::prototype_id(ctx, action_id).await?;
        let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
        let func_run_id = Action::last_func_run_id_for_id_opt(ctx, action_id).await?;

        let component_id = Action::component_id(ctx, action_id).await?;
        let (component_schema_name, component_name) = match component_id {
            Some(component_id) => {
                let schema = Component::schema_for_component_id(ctx, component_id).await?;
                let component_name = Component::name_by_id(ctx, component_id).await?;
                (Some(schema.name().to_owned()), Some(component_name))
            }
            None => (None, None),
        };
        let mut my_dependencies = action_graph.get_all_dependencies(action_id);
        my_dependencies.sort();
        let mut dependent_on = action_graph.direct_dependencies_of(action_id);
        dependent_on.sort();
        let mut hold_status_influenced_by =
            Action::get_hold_status_influenced_by(ctx, &action_graph, action_id).await?;
        hold_status_influenced_by.sort();
        views.push(ActionView {
            id: action_id,
            prototype_id: prototype.id(),
            name: prototype.name().clone(),
            component_id,
            component_schema_name,
            component_name,
            description: func.display_name,
            kind: prototype.kind.into(),
            state: action.state(),
            func_run_id,
            originating_change_set_id: action.originating_changeset_id(),
            my_dependencies,
            dependent_on,
            hold_status_influenced_by,
        })
    }
    let workspace_mv_id = ctx.workspace_pk()?;
    Ok(ActionViewListMv {
        id: workspace_mv_id,
        actions: views,
    })
}
