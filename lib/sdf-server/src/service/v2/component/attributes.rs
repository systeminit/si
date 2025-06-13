use std::collections::HashMap;

use axum::{
    Json,
    Router,
    extract::Path,
    routing::put,
};
use dal::{
    ChangeSet,
    attribute::attributes::{
        AttributeValueIdent,
        ValueOrSourceSpec,
    },
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde_json::json;

use super::{
    ComponentIdFromPath,
    Result,
};
use crate::app_state::AppState;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/", put(update_attributes))
}

async fn update_attributes(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(updates): Json<HashMap<AttributeValueIdent, ValueOrSourceSpec>>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let counts = dal::update_attributes(ctx, component_id, updates).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "component_attributes_updated",
        json!({
            "how": "/component/attributes",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
            "set_count": counts.set_count,
            "unset_count": counts.unset_count,
            "subscription_count": counts.subscription_count,
        }),
    );

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
