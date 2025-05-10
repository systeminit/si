use dal::{
    DalContext,
    diagram::view::View,
};
use si_frontend_mv_types::View as ViewMv;
use si_id::ViewId;
use telemetry::prelude::*;

#[instrument(name = "dal_materialized_views.view", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext, view_id: ViewId) -> super::Result<ViewMv> {
    let ctx = &ctx;
    let view = View::get_by_id(ctx, view_id).await?;
    let is_default = view.is_default(ctx).await?;

    Ok(ViewMv {
        id: view.id(),
        name: view.name().to_owned(),
        is_default,
        timestamp: view.timestamp().to_owned(),
    })
}
