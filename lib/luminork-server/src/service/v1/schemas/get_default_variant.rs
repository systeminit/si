use axum::extract::Path;
use dal::{
    Schema,
    cached_module::CachedModule,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_default_variant::CachedDefaultVariant,
    luminork_default_variant::LuminorkDefaultVariant,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    BuildingResponseV1,
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
    SchemaVariantFunc,
    SchemaVariantResponseV1,
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
        (status = 202, description = "Schema variant building, try again later", body = BuildingResponseV1),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn get_default_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(frigg): FriggStore,
    edda_client: EddaClient,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<SchemaVariantResponseV1> {
    match frigg
        .get_current_workspace_object(
            ctx.workspace_pk()?,
            ctx.change_set_id(),
            &ReferenceKind::LuminorkDefaultVariant.to_string(),
            &schema_id.to_string(),
        )
        .await
    {
        Ok(Some(obj)) => {
            if let Ok(luminork_default_variant) =
                serde_json::from_value::<LuminorkDefaultVariant>(obj.data)
            {
                // Clone values for logging before moving them
                let display_name_for_log = luminork_default_variant.display_name.clone();
                let category_for_log = luminork_default_variant.category.clone();

                let variant_funcs: Vec<SchemaVariantFunc> = luminork_default_variant
                    .variant_funcs
                    .into_iter()
                    .map(SchemaVariantFunc::from)
                    .collect();

                // We know it is a builtin if we find a CachedSchema for its schema id
                // The only cached schemas we currently build are builtins - if that changes, this logic will need to change!
                let installed_from_upstream = (frigg
                    .get_current_deployment_object(
                        ReferenceKind::CachedSchema,
                        &schema_id.to_string(),
                    )
                    .await?)
                    .is_some();

                let response = GetSchemaVariantV1Response {
                    variant_id: luminork_default_variant.variant_id,
                    display_name: luminork_default_variant.display_name,
                    category: luminork_default_variant.category,
                    color: luminork_default_variant.color,
                    is_locked: luminork_default_variant.is_locked,
                    installed_from_upstream,
                    description: luminork_default_variant.description,
                    link: luminork_default_variant.link,
                    asset_func_id: luminork_default_variant.asset_func_id,
                    variant_func_ids: luminork_default_variant.variant_func_ids,
                    variant_funcs,
                    is_default_variant: true, // Always true for default variant endpoint
                    domain_props: luminork_default_variant.domain_props.map(Into::into),
                };

                tracker.track(
                    ctx,
                    "api_get_default_variant",
                    json!({
                        "schema_id": schema_id,
                        "schema_variant_id": luminork_default_variant.variant_id,
                        "schema_variant_name": display_name_for_log,
                        "schema_variant_category": category_for_log,
                        "is_locked": luminork_default_variant.is_locked
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(response)));
            }
        }
        Ok(None) => {
            // LuminorkDefaultVariant MV not found - check if schema exists in graph (installed)
            if Schema::get_by_id_opt(ctx, schema_id).await?.is_some() {
                // Schema exists in graph but LuminorkDefaultVariant MV not built yet
                // Send edda rebuild request to trigger MV generation
                if let Err(e) = edda_client
                    .rebuild_for_change_set(ctx.workspace_pk()?, ctx.change_set_id())
                    .await
                {
                    warn!(
                        "Failed to send edda rebuild request for schema {}: {}",
                        schema_id, e
                    );
                }

                return Ok(SchemaVariantResponseV1::Building(BuildingResponseV1 {
                    status: "building".to_string(),
                    message: "Default variant data is being generated from workspace graph, please retry shortly".to_string(),
                    retry_after_seconds: 2,
                    estimated_completion_seconds: 5,
                }));
            }
        }
        Err(e) => {
            warn!(
                "Failed to get LuminorkDefaultVariant MV for schema {}: {}",
                schema_id, e
            );
            // Continue to fallback logic
        }
    }

    match frigg
        .get_current_deployment_object(ReferenceKind::CachedDefaultVariant, &schema_id.to_string())
        .await?
    {
        Some(obj) => {
            if let Ok(cached_default_variant) =
                serde_json::from_value::<CachedDefaultVariant>(obj.data)
            {
                // Clone values for logging before moving them
                let display_name_for_log = cached_default_variant.display_name.clone();
                let category_for_log = cached_default_variant.category.clone();

                let response = GetSchemaVariantV1Response {
                    variant_id: cached_default_variant.variant_id,
                    display_name: cached_default_variant.display_name,
                    category: cached_default_variant.category,
                    color: cached_default_variant.color,
                    is_locked: cached_default_variant.is_locked,
                    installed_from_upstream: true,
                    description: cached_default_variant.description,
                    link: cached_default_variant.link,
                    asset_func_id: cached_default_variant.asset_func_id,
                    variant_func_ids: cached_default_variant.variant_func_ids,
                    variant_funcs: vec![],
                    is_default_variant: true, // Always true for default variant endpoint
                    domain_props: cached_default_variant.domain_props.map(Into::into),
                };

                tracker.track(
                    ctx,
                    "api_get_default_variant",
                    json!({
                        "schema_id": schema_id,
                        "schema_variant_id": cached_default_variant.variant_id,
                        "schema_variant_name": display_name_for_log,
                        "schema_variant_category": category_for_log,
                        "is_locked": cached_default_variant.is_locked
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
                        message: "Default variant data is being generated from cached modules, please retry shortly".to_string(),
                        retry_after_seconds: 2,
                        estimated_completion_seconds: 10,
                    }));
                }
                Ok(None) => {
                    // Schema doesn't exist in cached_modules at all - return 404
                    return Err(SchemaError::SchemaNotFound(schema_id));
                }
                Err(e) => {
                    warn!(
                        "Failed to check cached module for schema {}: {}",
                        schema_id, e
                    );
                }
            }
        }
    }

    // Schema not found anywhere
    Err(SchemaError::SchemaNotFound(schema_id))
}
