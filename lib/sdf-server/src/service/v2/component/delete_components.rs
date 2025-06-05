use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    component::delete,
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;

use super::Result;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub force_erase: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<DeleteComponentsRequest>,
) -> Result<ForceChangeSetResponse<HashMap<ComponentId, bool>>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let mut result = HashMap::new();

    let mut track_payloads = vec![];
    // Schema names have to be gathered before deletion
    for &component_id in &request.component_ids {
        let component_schema_name = Component::schema_for_component_id(&ctx, component_id)
            .await?
            .name()
            .to_string();

        track_payloads.push(serde_json::json!({
            "how": "/diagram/delete_component",
            "component_id": component_id,
            "component_schema_name": component_schema_name,
            "change_set_id": ctx.change_set_id(),
        }));
    }

    for (component_id, status) in
        delete::delete_components(&ctx, &request.component_ids, request.force_erase).await?
    {
        result.insert(
            component_id,
            matches!(status, delete::ComponentDeletionStatus::MarkedForDeletion),
        );
    }

    for payload in track_payloads {
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "delete_component",
            payload,
        );
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, result))
}
