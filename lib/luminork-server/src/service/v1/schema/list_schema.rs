use std::collections::HashSet;

use axum::response::Json;
use dal::{
    Schema,
    SchemaId,
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

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schema",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
    ),
    tag = "schemas",
    responses(
        (status = 200, description = "Schemas listed successfully", body = ListSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_schemas(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
) -> Result<Json<ListSchemaV1Response>, SchemaError> {
    let mut schema_resp: Vec<SchemaResponse> = vec![];
    let schema_ids = Schema::list_ids(ctx).await?;
    let installed_schema_ids: HashSet<_> = schema_ids.iter().collect();
    let cached_modules = CachedModule::latest_modules(ctx).await?;
    for module in cached_modules {
        let is_installed = installed_schema_ids.contains(&module.schema_id);
        schema_resp.push(SchemaResponse {
            schema_name: module.schema_name,
            schema_id: module.schema_id,
            category: module.category,
            installed: is_installed,
        });
    }

    tracker.track(ctx, "api_list_schemas", json!({}));

    Ok(Json(ListSchemaV1Response {
        schemas: schema_resp,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaV1Response {
    #[schema(value_type = Vec<Object>, example = "[{\"schemaId\":\"01H9ZQD35JPMBGHH69BT0Q79VY\",\"schemaName\":\"AWS::EC2::Instance\",\"category\":\"AWS::EC2\",\"installed\": \"true\"}]")]
    pub schemas: Vec<SchemaResponse>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
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
