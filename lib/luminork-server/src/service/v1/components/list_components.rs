use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    Component,
    ComponentId,
};
use serde::Serialize;
use serde_json::json;
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
    #[deprecated(note = "component ids deprecated in favor of component details")]
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,

    #[schema(
        value_type = Vec<ComponentDetailsV1>,
        example = json!([
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                "name": "my-vpc",
                "schema_name": "AWS::EC2::VPC"
            },
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                "name": "Public 1",
                "schema_name": "AWS::EC2::Subnet"
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
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (ComponentId of the last item from previous page)"),
    ),
    summary = "List all components",
    tag = "components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = ListComponentsV1Response, example = json!({
                    "components": [
                        "01H9ZQD35JPMBGHH69BT0Q79AA",
                        "01H9ZQD35JPMBGHH69BT0Q79BB"
                    ],
                    "componentDetails": [
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                            "name": "my-vpc",
                            "schema_name": "AWS::EC2::VPC"
                        },
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                            "name": "Public 1",
                            "schema_name": "AWS::EC2::Subnet"
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

    let mut component_ids = Vec::with_capacity(limit);
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
        comp_details.push(ComponentDetailsV1 {
            component_id: component.id(),
            name,
            schema_name,
        });

        component_ids.push(component.id());
    }

    tracker.track(ctx, "api_list_components", json!({}));

    Ok(Json(ListComponentsV1Response {
        components: component_ids,
        component_details: comp_details,
        next_cursor,
    }))
}
