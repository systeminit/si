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

    

    Ok(ViewListMv {
        id: ctx.change_set_id(),
        views: view_ids.iter().map(|&id| id.into()).collect(),
    })
}
