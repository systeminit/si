use axum::{
    Json,
    extract::Path,
};
use dal::{
    Prop,
    Schema,
    SchemaVariant,
    schema::variant::root_prop::RootPropChild,
};
use serde_json::json;

use super::{
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
    build_prop_schema_tree,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/default",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
    ),
    tag = "schemas",
    summary = "Get the default variant for a schema id",
    responses(
        (status = 200, description = "Schema variant retrieved successfully", body = GetSchemaVariantV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]

pub async fn get_default_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<Json<GetSchemaVariantV1Response>> {
    let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;

    let variant = SchemaVariant::get_by_id_opt(ctx, default_variant_id)
        .await?
        .ok_or(SchemaError::SchemaVariantNotFound(default_variant_id))?;

    let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(ctx, default_variant_id)
        .await?
        .into_iter()
        .collect();

    let domain =
        Prop::find_prop_by_path(ctx, default_variant_id, &RootPropChild::Domain.prop_path())
            .await
            .map_err(Box::new)?;
    let domain_prop_schema = build_prop_schema_tree(ctx, domain.id).await?;

    tracker.track(
        ctx,
        "api_get_default_variant",
        json!({
            "schema_id": schema_id,
            "schema_variant_id": default_variant_id,
            "schema_variant_name": variant.display_name(),
            "schema_variant_category": variant.category(),
            "is_locked": variant.is_locked()
        }),
    );

    Ok(Json(GetSchemaVariantV1Response {
        variant_id: default_variant_id,
        display_name: variant.display_name().into(),
        category: variant.category().into(),
        color: variant.color().into(),
        is_locked: variant.is_locked(),
        description: variant.description(),
        link: variant.link(),
        asset_func_id: variant.asset_func_id_or_error()?,
        variant_func_ids,
        is_default_variant: true,
        domain_props: domain_prop_schema,
    }))
}
