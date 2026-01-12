use std::str::FromStr;

use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    Schema,
    SchemaVariant,
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
    #[schema(value_type = Option<String>)]
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

    let (schema_name, schema_id, category, installed) = match schema_ref {
        SchemaReference::ById { schema_id } => match Schema::get_by_id_opt(ctx, schema_id).await? {
            Some(schema) => {
                let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
                let default_variant = SchemaVariant::get_by_id(ctx, default_variant_id).await?;
                (
                    schema.name,
                    schema_id,
                    Some(default_variant.category().to_string()),
                    true,
                )
            }
            None => match CachedModule::find_latest_for_schema_id(ctx, schema_id).await? {
                Some(module) => {
                    let installed = Schema::exists_locally(ctx, module.schema_id).await?;
                    (
                        module.schema_name,
                        module.schema_id,
                        module.category,
                        installed,
                    )
                }
                None => {
                    tracker.track(
                        ctx,
                        "api_find_schema",
                        json!({
                            "search_term": schema_id.to_string(),
                            "search_type": "schema_id",
                            "not_found": "true"
                        }),
                    );
                    return Err(SchemaError::SchemaNotFound(schema_id));
                }
            },
        },
        SchemaReference::ByName {
            schema: schema_name,
        } => match Schema::get_by_name_opt(ctx, schema_name.as_str()).await? {
            Some(schema) => {
                let default_variant_id = Schema::default_variant_id(ctx, schema.id()).await?;
                let default_variant = SchemaVariant::get_by_id(ctx, default_variant_id).await?;
                (
                    schema_name,
                    schema.id(),
                    Some(default_variant.category().to_string()),
                    true,
                )
            }
            None => {
                match CachedModule::find_latest_for_schema_name(ctx, schema_name.as_str()).await? {
                    Some(module) => {
                        let installed = Schema::exists_locally(ctx, module.schema_id).await?;
                        (
                            module.schema_name,
                            module.schema_id,
                            module.category,
                            installed,
                        )
                    }
                    None => {
                        tracker.track(
                            ctx,
                            "api_find_schema",
                            json!({
                                "search_term": schema_name.clone(),
                                "search_type": "schema_name",
                                "not_found": "true"
                            }),
                        );
                        return Err(SchemaError::SchemaNotFoundByName(schema_name));
                    }
                }
            }
        },
    };

    tracker.track(
        ctx,
        "api_find_schema",
        json!({
            "schema_id": schema_id,
        }),
    );

    Ok(Json(FindSchemaV1Response {
        schema_name,
        schema_id,
        category,
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
    #[schema(value_type = Option<String>)]
    pub category: Option<String>,
    #[schema(value_type = bool)]
    pub installed: bool,
}

enum SchemaReference {
    ByName { schema: String },
    ById { schema_id: SchemaId },
}
