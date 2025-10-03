use axum::extract::Path;
use dal::{
    SchemaVariant,
    cached_module::CachedModule,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_default_variant::CachedDefaultVariant,
    luminork_schema_variant::LuminorkSchemaVariant,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    BuildingResponseV1,
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaVariantResponseV1,
    SchemaVariantV1RequestPath,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
    ),
    summary = "Get a schema variant by schema id and schema variant id",
    tag = "schemas",
    responses(
        (status = 200, description = "Schema variant retrieved successfully", body = GetSchemaVariantV1Response),
        (status = 202, description = "Schema variant building, try again later", body = BuildingResponseV1),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]

pub async fn get_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    frigg: FriggStore,
    edda_client: EddaClient,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
) -> SchemaResult<SchemaVariantResponseV1> {
    // Phase 1: Try LuminorkSchemaVariant MV first (change set-level, built from workspace graph)
    match frigg
        .get_current_workspace_object(
            ctx.workspace_pk()?,
            ctx.change_set_id(),
            &ReferenceKind::LuminorkSchemaVariant.to_string(),
            &schema_variant_id.to_string(),
        )
        .await?
    {
        Some(obj) => {
            if let Ok(luminork_variant) = serde_json::from_value::<LuminorkSchemaVariant>(obj.data)
            {
                // Clone values for logging before moving them
                let display_name_for_log = luminork_variant.display_name.clone();
                let category_for_log = luminork_variant.category.clone();

                let response = GetSchemaVariantV1Response {
                    variant_id: luminork_variant.variant_id,
                    display_name: luminork_variant.display_name,
                    category: luminork_variant.category,
                    color: luminork_variant.color,
                    is_locked: luminork_variant.is_locked,
                    description: luminork_variant.description,
                    link: luminork_variant.link,
                    asset_func_id: luminork_variant.asset_func_id,
                    variant_func_ids: luminork_variant.variant_func_ids,
                    is_default_variant: luminork_variant.is_default_variant,
                    domain_props: luminork_variant.domain_props.map(Into::into),
                };

                tracker.track(
                    ctx,
                    "api_get_variant",
                    json!({
                        "schema_id": schema_id,
                        "schema_variant_id": schema_variant_id,
                        "schema_variant_name": display_name_for_log,
                        "schema_variant_category": category_for_log,
                        "is_locked": luminork_variant.is_locked
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(response)));
            }
        }
        None => {
            // LuminorkSchemaVariant MV not found - check if schema variant exists in graph (installed)
            if SchemaVariant::get_by_id_opt(ctx, schema_variant_id)
                .await?
                .is_some()
            {
                // Schema variant exists in graph but LuminorkSchemaVariant MV not built yet
                // Send edda rebuild request to trigger MV generation
                if let Err(e) = edda_client
                    .rebuild_for_change_set(ctx.workspace_pk()?, ctx.change_set_id())
                    .await
                {
                    warn!(
                        "Failed to send edda rebuild request for schema variant {}: {}",
                        schema_variant_id, e
                    );
                }

                return Ok(SchemaVariantResponseV1::Building(BuildingResponseV1 {
                    status: "building".to_string(),
                    message: "Schema variant data is being generated from workspace graph, please retry shortly".to_string(),
                    retry_after_seconds: 2,
                    estimated_completion_seconds: 5,
                }));
            }
        }
    }

    // Phase 1 Fallback: Try CachedDefaultVariant MV (deployment-level, for uninstalled modules)
    match frigg
        .get_current_deployment_object(ReferenceKind::CachedDefaultVariant, &schema_id.to_string())
        .await?
    {
        Some(obj) => {
            if let Ok(cached_variant) = serde_json::from_value::<CachedDefaultVariant>(obj.data) {
                // Clone values for logging before moving them
                let display_name_for_log = cached_variant.display_name.clone();
                let category_for_log = cached_variant.category.clone();

                let response = GetSchemaVariantV1Response {
                    variant_id: cached_variant.variant_id,
                    display_name: cached_variant.display_name,
                    category: cached_variant.category,
                    color: cached_variant.color,
                    is_locked: cached_variant.is_locked,
                    description: cached_variant.description,
                    link: cached_variant.link,
                    asset_func_id: cached_variant.asset_func_id,
                    variant_func_ids: cached_variant.variant_func_ids,
                    is_default_variant: true, // CachedDefaultVariant is always the default variant for the schema
                    domain_props: cached_variant.domain_props.map(Into::into),
                };

                tracker.track(
                    ctx,
                    "api_get_variant",
                    json!({
                        "schema_id": schema_id,
                        "schema_variant_id": schema_variant_id,
                        "schema_variant_name": display_name_for_log,
                        "schema_variant_category": category_for_log,
                        "is_locked": cached_variant.is_locked
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(response)));
            }
        }
        None => {
            // CachedDefaultVariant not found - check if schema exists in cached_modules DB table
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists in cached_modules but CachedDefaultVariant MV not built yet
                    return Ok(SchemaVariantResponseV1::Building(BuildingResponseV1 {
                        status: "building".to_string(),
                        message: "Schema variant data is being generated from cached modules, please retry shortly".to_string(),
                        retry_after_seconds: 2,
                        estimated_completion_seconds: 10,
                    }));
                }
                Ok(None) => {
                    // Schema doesn't exist in cached_modules at all - return 404
                }
                Err(e) => {
                    warn!(
                        "Failed to check cached module for schema {}: {}",
                        schema_id, e
                    );
                    // Fall through to 404 if we can't check the DB
                }
            }
        }
    }

    // Schema variant not found anywhere
    Err(SchemaError::SchemaVariantNotFound(schema_variant_id))
}
