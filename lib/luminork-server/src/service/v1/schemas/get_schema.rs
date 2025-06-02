use axum::{
    Json,
    extract::Path,
};
use dal::{
    Schema,
    SchemaVariant,
};
use itertools::Itertools;
use serde_json::json;

use super::{
    GetSchemaV1Response,
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
    ),
    tag = "schemas",
    summary = "Get a schema by schema id",
    responses(
        (status = 200, description = "Schema retrieved successfully", body = GetSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<Json<GetSchemaV1Response>> {
    let schema = Schema::get_by_id_opt(ctx, schema_id)
        .await?
        .ok_or(SchemaError::SchemaNotFound(schema_id))?;

    let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
    let variants = SchemaVariant::list_for_schema(ctx, schema_id).await?;

    tracker.track(
        ctx,
        "api_get_schema",
        json!({
            "schema_id": schema_id,
            "schema_name": schema.name,
            "default_variant_id": default_variant_id
        }),
    );

    Ok(Json(GetSchemaV1Response {
        name: schema.name,
        default_variant_id,
        variant_ids: variants.into_iter().map(|v| v.id).collect_vec(),
    }))
}
