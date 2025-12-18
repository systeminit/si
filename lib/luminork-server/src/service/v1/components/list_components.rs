use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    ComponentId,
};
use serde::Serialize;
use serde_json::{
    Value,
    json,
};
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentsError;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::common::PaginationParams,
};

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    #[schema(
        value_type = Vec<ComponentDetailsV1>,
        example = json!([
            {
                "componentId": "01H9ZQD35JPMBGHH69BT0Q79AA",
                "name": "my-vpc",
                "schemaName": "AWS::EC2::VPC"
            },
            {
                "componentId": "01H9ZQD35JPMBGHH69BT0Q79BB",
                "name": "Public 1",
                "schemaName": "AWS::EC2::Subnet",
                "qualificationStatus": {
                    "total": 2,
                    "succeeded": 1,
                    "warned": 0,
                    "failed": 0,
                    "running": 1
                },
                "canBeUpgraded": true
            }
        ])
    )]
    pub component_details: Vec<ComponentDetailsV1>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDetailsV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub name: String,
    pub schema_name: String,
    pub codegen: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualification_status: Option<QualificationStatusV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_be_upgraded: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct QualificationStatusV1 {
    pub total: u64,
    pub succeeded: u64,
    pub warned: u64,
    pub failed: u64,
    pub running: u64,
}

impl From<si_frontend_types::ComponentQualificationStats> for QualificationStatusV1 {
    fn from(stats: si_frontend_types::ComponentQualificationStats) -> Self {
        Self {
            total: stats.total,
            succeeded: stats.succeeded,
            warned: stats.warned,
            failed: stats.failed,
            running: stats.running,
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (ComponentId of the last item from previous page)"),
        ("includeCodegen" = Option<bool>, Query, description = "Allow returning the codegen for the cloudformation template for the component (if it exists)"),
        ("includeQualifications" = Option<bool>, Query, description = "Include real-time qualification status"),
        ("includeUpgradeStatus" = Option<bool>, Query, description = "Include upgrade-ability information"),
    ),
    summary = "List all components",
    tag = "components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = ListComponentsV1Response, example = json!({
                    "componentDetails": [
                        {
                            "componentId": "01H9ZQD35JPMBGHH69BT0Q79AA",
                            "name": "my-vpc",
                            "schemaName": "AWS::EC2::VPC"
                        },
                        {
                            "componentId": "01H9ZQD35JPMBGHH69BT0Q79BB",
                            "name": "Public 1",
                            "schemaName": "AWS::EC2::Subnet",
                            "qualificationStatus": {
                                "total": 2,
                                "succeeded": 1,
                                "warned": 0,
                                "failed": 0,
                                "running": 1
                            },
                            "canBeUpgraded": true
                        }
                    ],
                    "nextCursor": null
                })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn list_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<PaginationParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListComponentsV1Response>, ComponentsError> {
    // Set default limit and enforce a max limit
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;

    let mut comp_details = Vec::with_capacity(limit);

    // Get all component
    let mut all_components = Component::list(ctx).await?;

    // Sort components  for consistent pagination
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

        let mut comp_response = ComponentDetailsV1 {
            component_id: component.id(),
            name,
            schema_name,
            codegen: None,
            qualification_status: None,
            can_be_upgraded: None,
        };

        // Handle existing includeCodegen parameter for backward compatibility
        if let Some(true) = params.include_codegen {
            let code_map_av_id =
                Component::find_code_map_attribute_value_id(ctx, component.id()).await?;

            let view = AttributeValue::view(ctx, code_map_av_id).await?;
            if let Some(v) = view {
                let details = v.get("awsCloudFormationLint");
                comp_response.codegen = details.cloned();
            }
        }

        // Handle new inclusion parameters
        if let Some(true) = params.include_qualifications {
            let stats =
                super::get_qualification_stats_with_realtime_running(ctx, component.id()).await?;
            comp_response.qualification_status = Some(stats);
        }

        if let Some(true) = params.include_upgrade_status {
            comp_response.can_be_upgraded = Some(component.can_be_upgraded(ctx).await?);
        }

        comp_details.push(comp_response);
    }

    tracker.track(ctx, "api_list_components", json!({}));

    Ok(Json(ListComponentsV1Response {
        component_details: comp_details,
        next_cursor,
    }))
}
