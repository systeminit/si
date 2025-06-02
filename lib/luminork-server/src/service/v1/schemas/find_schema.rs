use std::str::FromStr;

use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    Schema,
    cached_module::CachedModule,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_id::SchemaId;
use utoipa::{
    self,
    IntoParams,
    ToSchema,
};

use super::{
    SchemaError,
    SchemaResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Deserialize, Serialize, Debug, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
#[into_params(style = Form, parameter_in = Query)]
pub struct FindSchemaV1Params {
    #[param(required = false, nullable = true)]
    pub schema: Option<String>,

    #[serde(rename = "schemaId")]
    #[param(value_type = String, required = false, nullable = true)]
    #[schema(value_type = String)]
    pub schema_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/find",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        FindSchemaV1Params,
    ),
    tag = "schemas",
    summary = "Find schema by name or schema id",
    responses(
        (status = 200, description = "Schema retrieved successfully", body = FindSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn find_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Query(params): Query<FindSchemaV1Params>,
) -> SchemaResult<Json<FindSchemaV1Response>> {
    let schema_ref = if let Some(schema_id_str) = &params.schema_id {
        let schema_id = SchemaId::from_str(schema_id_str)?;
        SchemaReference::ById { schema_id }
    } else if let Some(schema_name) = &params.schema {
        SchemaReference::ByName {
            schema: schema_name.clone(),
        }
    } else {
        return Err(SchemaError::Validation(
            "Either schema or schemaId must be provided".to_string(),
        ));
    };

    let module = match schema_ref {
        SchemaReference::ById { schema_id } => {
            if let Some(module) = CachedModule::find_latest_for_schema_id(ctx, schema_id).await? {
                module
            } else {
                return Err(SchemaError::SchemaNotFound(schema_id));
            }
        }
        SchemaReference::ByName { schema } => {
            if let Some(module) =
                CachedModule::find_latest_for_schema_name(ctx, schema.as_str()).await?
            {
                module
            } else {
                return Err(SchemaError::SchemaNameNotFound(schema));
            }
        }
    };

    let installed = Schema::exists_locally(ctx, module.schema_id).await?;

    tracker.track(
        ctx,
        "api_find_schema",
        json!({
            "schema_id": module.schema_id
        }),
    );

    Ok(Json(FindSchemaV1Response {
        schema_name: module.schema_name,
        schema_id: module.schema_id,
        category: module.category,
        installed,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FindSchemaV1Response {
    #[schema(value_type = String)]
    pub schema_name: String,
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub category: Option<String>,
    #[schema(value_type = bool)]
    pub installed: bool,
}

enum SchemaReference {
    ByName { schema: String },
    ById { schema_id: SchemaId },
}
