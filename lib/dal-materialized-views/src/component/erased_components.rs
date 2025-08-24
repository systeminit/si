use std::collections::{
    HashMap,
    HashSet,
};

use dal::{
    Component,
    ComponentId,
    DalContext,
};
use si_frontend_mv_types::component::erased_components::{
    ErasedComponents,
    HeadComponent,
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.component_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(new_ctx: DalContext) -> crate::Result<ErasedComponents> {
    let new_ctx = &new_ctx;
    let old_ctx = new_ctx.clone_with_head().await?;
    let old_ctx = &old_ctx;
    if new_ctx.change_set_id() == old_ctx.change_set_id() {
        return Ok(ErasedComponents {
            id: new_ctx.workspace_pk()?,
            erased: HashMap::new(),
        });
    }
    // Get all of the component Ids on head, and in this change set
    let old_ids: HashSet<ComponentId> =
        HashSet::from_iter(Component::list_ids(old_ctx).await?.iter().copied());
    let new_ids: HashSet<ComponentId> =
        HashSet::from_iter(Component::list_ids(new_ctx).await?.iter().copied());

    let only_in_old: HashSet<&ComponentId> = old_ids.difference(&new_ids).collect();
    let mut erased = HashMap::new();
    for &component_id in only_in_old {
        let diff = super::component_diff::assemble(new_ctx.clone(), component_id).await?;
        let component = crate::component::assemble_in_list(old_ctx.clone(), component_id).await?;

        erased.insert(component_id, HeadComponent { diff, component });
    }

    let workspace_mv_id = new_ctx.workspace_pk()?;
    Ok(ErasedComponents {
        id: workspace_mv_id,
        erased,
    })
}
