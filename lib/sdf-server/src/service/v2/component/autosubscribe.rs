use axum::Json;
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    component::suggestion::{
        ConflictedSubscription,
        SuccessfulSubscription,
    },
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

use super::Result;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutosubscribeComponentRequest {
    pub component_id: ComponentId,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutosubscribeComponentResponse {
    pub created_subscriptions: Vec<SuccessfulSubscription>,
    pub conflicts: Vec<ConflictedSubscription>,
}

/// Run autosubscribe for a component to automatically create prop subscriptions based on prop suggestions
pub async fn autosubscribe_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<AutosubscribeComponentRequest>,
) -> Result<ForceChangeSetResponse<AutosubscribeComponentResponse>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let component_schema_name = Component::schema_for_component_id(ctx, request.component_id)
        .await?
        .name()
        .to_string();

    let auto_subscribe_result = Component::autosubscribe(ctx, request.component_id).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "autosubscribe_component",
        serde_json::json!({
            "how": "/components/autosubscribe",
            "component_id": request.component_id,
            "component_schema_name": component_schema_name,
            "subscriptions_created": auto_subscribe_result.success_count() ,
            "conflicts_found": auto_subscribe_result.conflict_count(),
            "change_set_id": ctx.change_set_id(),
        }),
    );
    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        AutosubscribeComponentResponse {
            created_subscriptions: auto_subscribe_result.successful,
            conflicts: auto_subscribe_result.conflicts,
        },
    ))
}
