use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Schema,
    SchemaVariant,
};
use itertools::Itertools;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde_json::json;
use utoipa::{
    self,
};

use super::{
    GetSchemaV1Response,
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/install",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
    ),
    tag = "schemas",
    summary = "Installs a schema - if there's an installed schema, it will return that schema detail",
    responses(
        (status = 200, description = "Schema installed successfully", body = GetSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn install_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<Json<GetSchemaV1Response>> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema = if let Some(installed_schema) = Schema::get_by_id_opt(ctx, schema_id).await? {
        installed_schema
    } else {
        Schema::get_or_install_default_variant(ctx, schema_id).await?;
        Schema::get_by_id_opt(ctx, schema_id)
            .await?
            .ok_or(SchemaError::SchemaNotFound(schema_id))?
    };

    tracker.track(
        ctx,
        "api_install_schema",
        json!({
            "schema_id": schema.id(),
        }),
    );

    ctx.commit().await?;

    let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
    let variants = SchemaVariant::list_for_schema(ctx, schema_id).await?;

    Ok(Json(GetSchemaV1Response {
        schema_id,
        name: schema.name,
        default_variant_id,
        variant_ids: variants.into_iter().map(|v| v.id).collect_vec(),
    }))
}
