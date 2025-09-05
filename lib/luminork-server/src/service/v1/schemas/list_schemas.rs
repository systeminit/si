use std::collections::{
    HashMap,
    HashSet,
};

use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    Schema,
    SchemaId,
    SchemaVariant,
    cached_module::CachedModule,
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

use super::SchemaError;
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaParams {
    #[schema(example = "50", nullable = true, value_type = Option<String>)]
    pub limit: Option<u32>,
    #[schema(example = "01H9ZQD35JPMBGHH69BT0Q79VY", nullable = true, value_type = Option<String>)]
    pub cursor: Option<String>,
    #[schema(example = "Templates", nullable = true, value_type = Option<String>)]
    pub category: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (SchemaId of the last item from previous page)"),
        ("category" = Option<String>, Query, description = "Category filter for schemas"),
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
    Query(params): Query<ListSchemaParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListSchemaV1Response>, SchemaError> {
    // Set default limit and enforce a max limit
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;
    let maybe_category_filter = params.category;

    // Get all installed schema IDs
    let schema_ids = Schema::list_ids(ctx).await?;
    let installed_schema_ids: HashSet<_> = schema_ids.iter().collect();

    // Get cached modules with their metadata
    let cached_modules = CachedModule::latest_modules(ctx).await?;
    // Create a map of schema ID to cached module data
    let mut cached_module_map: HashMap<SchemaId, CachedModule> = HashMap::new();
    for module in cached_modules {
        cached_module_map.insert(module.schema_id, module);
    }

    // Combine both sources to create a complete list
    let mut all_schemas: Vec<SchemaResponse> = Vec::new();
    // First add installed schemas from Schema::list_ids
    for schema_id in &schema_ids {
        if let Some(module) = cached_module_map.get(schema_id) {
            match (maybe_category_filter.as_deref(), module.category.as_deref()) {
                (Some(category_filter), Some(module_category))
                    if category_filter == module_category => {}
                (None, _) => {}
                _ => continue,
            }
            // Schema is both installed and in cache
            all_schemas.push(SchemaResponse {
                schema_name: module.schema_name.clone(),
                schema_id: *schema_id,
                category: module.category.clone(),
                installed: true,
            });
        } else {
            // Schema is installed but not in cache - this is a local only schema
            if let Ok(schema) = Schema::get_by_id(ctx, *schema_id).await {
                let default_variant = SchemaVariant::default_for_schema(ctx, *schema_id).await?;
                match (maybe_category_filter.as_deref(), default_variant.category()) {
                    (Some(category_filter), default_variant_category)
                        if category_filter == default_variant_category => {}
                    (None, _) => {}
                    _ => continue,
                }
                all_schemas.push(SchemaResponse {
                    schema_name: schema.name,
                    schema_id: *schema_id,
                    category: Some(default_variant.category().to_owned()),
                    installed: true,
                });
            }
        }

        cached_module_map.remove(schema_id);
    }

    // Now add remaining cached modules (uninstalled ones)
    for (schema_id, module) in cached_module_map {
        let is_installed = installed_schema_ids.contains(&schema_id);
        all_schemas.push(SchemaResponse {
            schema_name: module.schema_name,
            schema_id,
            category: module.category,
            installed: is_installed,
        });
    }

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

// Removed OpenAPI schema to fix stack overflow

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaResponse {
    #[schema(example = "AWS::EC2::Instance")]
    pub schema_name: String,
    #[schema(example = "AWS::EC2")]
    pub category: Option<String>,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub schema_id: SchemaId,
    #[schema(value_type = bool, example = "false")]
    pub installed: bool,
}
