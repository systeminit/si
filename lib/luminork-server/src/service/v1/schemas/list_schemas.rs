use axum::{
    extract::Query,
    response::Json,
};
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
    SchemaError,
    SchemaResponse,
    get_full_schema_list,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::common::PaginationParams,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (SchemaId of the last item from previous page)"),
    ),
    summary = "List all schemas (paginated endpoint)",
    tag = "schemas",
    responses(
        (status = 200, description = "Schemas listed successfully", body = ListSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_schemas(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<PaginationParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListSchemaV1Response>, SchemaError> {
    // Set default limit and enforce a max limit
    let min = params.limit.unwrap_or(50).min(300) as usize;
    let limit = min;
    let cursor = params.cursor;

    let mut all_schemas = get_full_schema_list(ctx).await?;

    // Sort schemas by schema_id for consistent pagination
    all_schemas.sort_by(|a, b| a.schema_id.cmp(&b.schema_id));
    let start_index = if let Some(ref cursor_str) = cursor {
        // Find the index of the cursor schema based on string comparison
        match all_schemas
            .iter()
            .position(|schema| schema.schema_id.to_string() == *cursor_str)
        {
            Some(index) => index + 1, // Start after the cursor
            None => 0,                // cursor is gone, start from beginning
        }
    } else {
        0 // Start from the beginning
    };

    // Get paginated schemas
    let end_index = (start_index + limit).min(all_schemas.len());
    let paginated_schemas: Vec<SchemaResponse> = all_schemas[start_index..end_index].to_vec();

    let next_cursor = if end_index < all_schemas.len() && !paginated_schemas.is_empty() {
        paginated_schemas
            .last()
            .map(|schema| schema.schema_id.to_string())
    } else {
        None
    };

    tracker.track(ctx, "api_list_schemas", json!({}));

    Ok(Json(ListSchemaV1Response {
        schemas: paginated_schemas,
        next_cursor,
    }))
}

// For runtime
#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaV1Response {
    pub schemas: Vec<SchemaResponse>,
    pub next_cursor: Option<String>,
}
