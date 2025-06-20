use axum::{
    Json,
    Router,
    extract::Path,
    routing::post,
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    ComponentIdFromPath,
    Result,
};
use crate::app_state::AppState;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/", post(manage_component))
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManageComponentRequest {
    pub component_id: ComponentId,
}

async fn manage_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(payload): Json<ManageComponentRequest>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    Component::manage_component(ctx, component_id, payload.component_id).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "manage_component",
        serde_json::json!({
            "how": "/component/manage",
            "manager_component_id": component_id,
            "managed_component_id": payload.component_id,
        }),
    );

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
