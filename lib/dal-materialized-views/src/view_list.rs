use dal::{
    DalContext,
    diagram::view::View,
};
use si_frontend_mv_types::ViewList as ViewListMv;
use telemetry::prelude::*;

#[instrument(name = "dal_materialized_views.view_list", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext) -> super::Result<ViewListMv> {
    let ctx = &ctx;
    let mut view_ids = View::list_ids(ctx).await?;
    view_ids.sort();

    let mut views = Vec::with_capacity(view_ids.len());
    for view_id in view_ids {
        views.push(super::view::assemble(ctx.clone(), view_id).await?);
    }
    let workspace_mv_id = ctx.workspace_pk()?;

    Ok(ViewListMv {
        id: workspace_mv_id,
        views: views.iter().map(Into::into).collect(),
    })
}
