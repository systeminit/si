use axum::{
    extract::Query,
    response::Json,
};
use dal::Component;
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_frontend_types::ComponentQualificationStats;
use si_id::ComponentId;
use utoipa::{self, ToSchema};

use super::ComponentsError;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::common::PaginationParams,
};

use dal::qualification::QualificationSummary;

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentInfoV1Response {
    #[schema(
        value_type = Vec<ComponentInfoV1>,
        example = json!([
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                "name": "my-vpc",
                "schema_name": "AWS::EC2::VPC",
                "qualification_status": {
                    "total": 3,
                    "succeeded": 2,
                    "warned": 0,
                    "failed": 1
                },
                "can_be_upgraded": true
            },
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                "name": "Public 1",
                "schema_name": "AWS::EC2::Subnet",
                "qualification_status": {
                    "total": 2,
                    "succeeded": 2,
                    "warned": 0,
                    "failed": 0
                },
                "can_be_upgraded": false
            }
        ])
    )]
    pub components: Vec<ComponentInfoV1>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfoV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub name: String,
    pub schema_name: String,
    pub qualification_status: QualificationStatusV1,
    pub can_be_upgraded: bool,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QualificationStatusV1 {
    pub total: u64,
    pub succeeded: u64,
    pub warned: u64,
    pub failed: u64,
}

impl From<ComponentQualificationStats> for QualificationStatusV1 {
    fn from(stats: ComponentQualificationStats) -> Self {
        Self {
            total: stats.total,
            succeeded: stats.succeeded,
            warned: stats.warned,
            failed: stats.failed,
        }
    }
}


#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/component-info",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (ComponentId of the last item from previous page)"),
    ),
    summary = "List all components with their qualification status and upgrade information",
    tag = "components",
    responses(
        (status = 200, description = "Component information retrieved successfully", body = ListComponentInfoV1Response, example = json!({
            "components": [
                {
                    "componentId": "01H9ZQD35JPMBGHH69BT0Q79AA",
                    "name": "my-vpc",
                    "schemaName": "AWS::EC2::VPC",
                    "qualificationStatus": {
                        "total": 3,
                        "succeeded": 2,
                        "warned": 0,
                        "failed": 1
                    },
                    "canBeUpgraded": true
                },
                {
                    "componentId": "01H9ZQD35JPMBGHH69BT0Q79BB",
                    "name": "Public 1",
                    "schemaName": "AWS::EC2::Subnet",
                    "qualificationStatus": {
                        "total": 2,
                        "succeeded": 2,
                        "warned": 0,
                        "failed": 0
                    },
                    "canBeUpgraded": false
                }
            ],
            "nextCursor": null
        })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_component_info(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<PaginationParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListComponentInfoV1Response>, ComponentsError> {
    // Set default limit and enforce a max limit
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;

    let mut components = Vec::with_capacity(limit);

    // Get all components
    let mut all_components = Component::list(ctx).await?;

    // Sort components for consistent pagination
    all_components.sort_by_key(|c| c.id());

    // Find the start index by matching the stringified ComponentId
    let start_index = if let Some(ref cursor_str) = cursor {
        match all_components
            .iter()
            .position(|component| component.id().to_string() == *cursor_str)
        {
            Some(index) => index + 1, // Start after the cursor
            None => 0,
        }
    } else {
        0 // Start from the beginning
    };

    // Compute the end index and extract the paginated slice
    let end_index = (start_index + limit).min(all_components.len());
    let paginated_components: Vec<Component> = all_components[start_index..end_index].to_vec();

    // Generate the next cursor from the last item's ID
    let next_cursor = if end_index < all_components.len() && !paginated_components.is_empty() {
        paginated_components
            .last()
            .map(|component| component.id().to_string())
    } else {
        None
    };

    for component in &paginated_components {
        let name = component.name(ctx).await?;
        let schema_name = component.schema(ctx).await?.name;
        let qualification_status = QualificationSummary::individual_stats(ctx, component.id())
            .await?
            .into();
        let can_be_upgraded = component.can_be_upgraded(ctx).await?;

        components.push(ComponentInfoV1 {
            component_id: component.id(),
            name,
            schema_name,
            qualification_status,
            can_be_upgraded,
        });
    }

    tracker.track(ctx, "api_list_component_info", json!({}));

    Ok(Json(ListComponentInfoV1Response {
        components,
        next_cursor,
    }))
}