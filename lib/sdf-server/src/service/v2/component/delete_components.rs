use std::collections::HashMap;

use axum::Json;
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    component::delete,
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
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub force_erase: bool,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<DeleteComponentsRequest>,
) -> Result<ForceChangeSetResponse<HashMap<ComponentId, bool>>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let mut result = HashMap::new();

    let mut track_payloads = vec![];
    for &component_id in &request.component_ids {
        let component_schema_name = Component::schema_for_component_id(ctx, component_id)
            .await?
            .name()
            .to_string();

        track_payloads.push(serde_json::json!({
            "how": "/diagram/delete_component",
            "erase": request.force_erase,
            "component_id": component_id,
            "component_schema_name": component_schema_name,
            "change_set_id": ctx.change_set_id(),
        }));
    }

    for (component_id, status) in
        delete::delete_components(ctx, &request.component_ids, request.force_erase).await?
    {
        result.insert(
            component_id,
            matches!(status, delete::ComponentDeletionStatus::MarkedForDeletion),
        );
    }

    for payload in track_payloads {
        tracker.track(ctx, "delete_component", payload);
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, result))
}
