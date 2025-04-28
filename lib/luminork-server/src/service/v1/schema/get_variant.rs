use axum::{
    Json,
    extract::Path,
};
use dal::{
    Schema,
    SchemaVariant,
};

use super::{
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaVariantV1RequestPath,
};
use crate::extract::change_set::ChangeSetDalContext;

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}/variant/{schema_variant_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("schema_id", description = "Schema identifier"),
        ("schema_variant_id", description = "Schema variant identifier"),
    ),
    tag = "schemas",
    responses(
        (status = 200, description = "Schema variant retrieved successfully", body = GetSchemaVariantV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
) -> SchemaResult<Json<GetSchemaVariantV1Response>> {
    let schema_variants = Schema::list_schema_variant_ids(ctx, schema_id).await?;
    if !schema_variants.contains(&schema_variant_id) {
        return Err(SchemaError::SchemaVariantNotMemberOfSchema(
            schema_id,
            schema_variant_id,
        ));
    }

    let variant = SchemaVariant::get_by_id_opt(ctx, schema_variant_id)
        .await?
        .ok_or(SchemaError::SchemaVariantNotFound(schema_variant_id))?;

    let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(ctx, schema_variant_id)
        .await?
        .into_iter()
        .collect();

    Ok(Json(GetSchemaVariantV1Response {
        variant_id: schema_variant_id,
        display_name: variant.display_name().into(),
        category: variant.category().into(),
        color: variant.color().into(),
        is_locked: variant.is_locked(),
        description: variant.description(),
        link: variant.link(),
        asset_func_id: variant.asset_func_id_or_error()?,
        variant_func_ids,
        is_default_variant: SchemaVariant::is_default_by_id(ctx, schema_variant_id).await?,
    }))
}
