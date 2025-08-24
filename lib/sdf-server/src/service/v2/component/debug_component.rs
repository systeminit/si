use axum::{
    Json,
    extract::Path,
};
use dal::component::debug::ComponentDebugView;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

use super::Result;
use crate::service::v2::component::ComponentIdFromPath;

pub(crate) async fn debug_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
) -> Result<Json<ComponentDebugView>> {
    let debug_view = ComponentDebugView::new(ctx, component_id).await?;

    tracker.track(
        ctx,
        "autosubscribe_component",
        serde_json::json!({
            "how": "/component/debug",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );
    Ok(Json(debug_view))
}
