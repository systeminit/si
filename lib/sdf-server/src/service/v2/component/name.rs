use axum::{
    Json,
    Router,
    extract::Path,
    routing::put,
};
use dal::{
    ChangeSet,
    Component,
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
use serde_json::json;

use super::{
    ComponentIdFromPath,
    Result,
};
use crate::app_state::AppState;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/", put(set_name))
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub name: String,
}

async fn set_name(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(payload): Json<SetNameRequest>,
) -> Result<ForceChangeSetResponse<()>> {
    // only in use by the new UI, no WsEvents needed!
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let component = Component::get_by_id(ctx, component_id).await?;
    component.set_name(ctx, &payload.name).await?;
    ctx.commit().await?;

    tracker.track(
        ctx,
        "component_set_name",
        json!({
            "how": "/component/set_name",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
