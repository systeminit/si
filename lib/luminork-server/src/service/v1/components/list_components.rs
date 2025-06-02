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

// For runtime
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    #[schema(value_type = Vec<Vec<String>>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub components: Vec<ComponentId>,
    pub next_cursor: Option<String>,
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
        (status = 200, description = "Components retrieved successfully", body = ListComponentsV1Response, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"])),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<PaginationParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListComponentsV1Response>, ComponentsError> {
    // Set default limit and enforce a max limit
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;

    // Get all component IDs
    let mut all_component_ids = Component::list_ids(ctx).await?;

    // Sort component IDs for consistent pagination
    all_component_ids.sort();

    // Apply cursor-based pagination with proper type handling
    let start_index = if let Some(ref cursor_str) = cursor {
        // Find the index of the cursor component - need to match on string representation
        match all_component_ids
            .iter()
            .position(|id| id.to_string() == *cursor_str)
        {
            Some(index) => index + 1, // Start after the cursor
            None => 0,
        }
    } else {
        0 // Start from the beginning
    };

    // Get paginated component IDs
    let end_index = (start_index + limit).min(all_component_ids.len());
    let paginated_component_ids: Vec<ComponentId> =
        all_component_ids[start_index..end_index].to_vec();

    // For the next cursor, return as a string
    let next_cursor = if end_index < all_component_ids.len() && !paginated_component_ids.is_empty()
    {
        paginated_component_ids.last().map(|id| id.to_string())
    } else {
        None
    };

    tracker.track(ctx, "api_list_components", json!({}));

    Ok(Json(ListComponentsV1Response {
        components: paginated_component_ids,
        next_cursor,
    }))
}
