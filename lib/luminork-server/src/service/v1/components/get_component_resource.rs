use axum::{
    Json,
    extract::Path,
};
use chrono::{
    DateTime,
    Utc,
};
use dal::Component;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentV1RequestPath,
    ComponentsError,
    ComponentsResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/resource",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Get a component resource by component Id",
    responses(
        (status = 200, description = "Component resource retrieved successfully", body = GetComponentResourceDataV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 412, description = "Component has no associated resource"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_component_resource(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> ComponentsResult<Json<GetComponentResourceDataV1Response>> {
    let resource = Component::resource_by_id(ctx, component_id)
        .await?
        .ok_or(ComponentsError::ComponentHasNoResource(component_id))?;

    tracker.track(
        ctx,
        "api_get_component_resource",
        json!({
            "component_id": component_id
        }),
    );

    Ok(Json(GetComponentResourceDataV1Response {
        status: resource.status,
        payload: resource.payload,
        last_synced: resource.last_synced,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
pub struct GetComponentResourceDataV1Response {
    #[schema(value_type = String, example = "Ok")]
    pub status: dal::component::resource::ResourceStatus,
    #[schema(example = json!{(
        resourceVal1: "value1",
        resourceVal2: "value2"
    )}, nullable = true)]
    pub payload: Option<serde_json::Value>,
    #[schema(value_type = String, format = DateTime, example = "2024-01-15T12:30:00Z")]
    pub last_synced: DateTime<Utc>,
}
