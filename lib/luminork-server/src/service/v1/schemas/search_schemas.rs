use axum::response::Json;
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
    SchemaResult,
    get_full_schema_list,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/search",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "schemas",
    request_body = SearchSchemasV1Request,
    summary = "Complex search for shemas",
    responses(
        (status = 200, description = "Schemas retrieved successfully", body = SearchSchemasV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn search_schemas(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<SearchSchemasV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<SearchSchemasV1Response>, SchemaError> {
    let Json(payload) = payload?;

    let mut all_schemas = get_full_schema_list(ctx).await?;

    if let Some(category) = payload.category.clone() {
        all_schemas = apply_category_filter(all_schemas, category).await?;
    }

    tracker.track(
        ctx,
        "api_search_schemas",
        json!({
            "category": payload.category,
        }),
    );

    Ok(Json(SearchSchemasV1Response {
        schemas: all_schemas,
    }))
}

async fn apply_category_filter(
    schemas: Vec<SchemaResponse>,
    category: String,
) -> SchemaResult<Vec<SchemaResponse>> {
    let mut filtered_schemas = Vec::new();

    for schema in schemas {
        if schema.category == Some(category.clone()) {
            filtered_schemas.push(schema);
        }
    }

    Ok(filtered_schemas)
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchSchemasV1Request {
    #[schema(example = "AWS::EC2", required = false)]
    pub category: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchSchemasV1Response {
    pub schemas: Vec<SchemaResponse>,
}
