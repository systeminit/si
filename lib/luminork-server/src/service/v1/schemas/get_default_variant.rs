use axum::extract::Path;
use dal::{
    Prop,
    Schema,
    SchemaVariant,
    cached_module::CachedModule,
    schema::variant::root_prop::RootPropChild,
    workspace_snapshot::traits::prop::PropExt,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_default_variant::CachedDefaultVariant,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    BuildingResponseV1,
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
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

pub async fn get_default_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(frigg): FriggStore,
    edda_client: EddaClient,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<SchemaVariantResponseV1> {
    // Try DAL lookup first (installed schema)
    if let Ok(default_variant_id) = Schema::default_variant_id(ctx, schema_id).await {
        if let Ok(Some(variant)) = SchemaVariant::get_by_id_opt(ctx, default_variant_id).await {
            let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(ctx, default_variant_id)
                .await?
                .into_iter()
                .collect();

            let domain = Prop::find_prop_by_path(
                ctx,
                default_variant_id,
                &RootPropChild::Domain.prop_path(),
            )
            .await
            .map_err(Box::new)?;
            let domain_prop_schema = ctx
                .workspace_snapshot()?
                .build_prop_schema_tree(ctx, domain.id)
                .await?
                .into();

            tracker.track(
                ctx,
                "api_get_default_variant",
                json!({
                    "schema_id": schema_id,
                    "schema_variant_id": default_variant_id,
                    "schema_variant_name": variant.display_name(),
                    "schema_variant_category": variant.category(),
                    "is_locked": variant.is_locked(),
                    "source": "dal"
                }),
            );

            return Ok(SchemaVariantResponseV1::Success(Box::new(
                GetSchemaVariantV1Response {
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
                    domain_props: Some(domain_prop_schema),
                },
            )));
        }
    }

    // Fall back to MV lookup for uninstalled schemas using direct CachedDefaultVariant lookup
    match frigg
        .get_current_deployment_object(ReferenceKind::CachedDefaultVariant, &schema_id.to_string())
        .await
    {
        Ok(Some(obj)) => {
            if let Ok(cached_default_variant) =
                serde_json::from_value::<CachedDefaultVariant>(obj.data)
            {
                // For uninstalled variants, domain props don't exist in DAL, so we set them to None
                // This matches the behavior in get_variant endpoint for cached variants

                // Clone values for logging before moving them
                let display_name_for_log = cached_default_variant.display_name.clone();
                let category_for_log = cached_default_variant.category.clone();

                let response = GetSchemaVariantV1Response {
                    variant_id: cached_default_variant.variant_id,
                    display_name: cached_default_variant.display_name,
                    category: cached_default_variant.category,
                    color: cached_default_variant.color,
                    is_locked: cached_default_variant.is_locked,
                    description: cached_default_variant.description,
                    link: cached_default_variant.link,
                    asset_func_id: cached_default_variant.asset_func_id,
                    variant_func_ids: cached_default_variant.variant_func_ids,
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
                        "is_locked": cached_default_variant.is_locked,
                        "source": "materialized_view"
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(response)));
            }
        }
        Ok(None) => {
            // CachedDefaultVariant not found - check if schema exists in cached_modules DB table
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists in cached_modules but CachedDefaultVariant MV not built yet - trigger rebuild and return 202
                    if let Err(e) = edda_client.rebuild_for_deployment().await {
                        warn!("Failed to trigger MV rebuild: {}", e);
                    }
                    let building_response = BuildingResponseV1 {
                        status: "building".to_string(),
                        message: "Schema variant data is being generated, please retry shortly"
                            .to_string(),
                        retry_after_seconds: 2,
                        estimated_completion_seconds: 10,
                    };
                    return Ok(SchemaVariantResponseV1::Building(building_response));
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
                    // Fall through to 404 if we can't check the DB
                }
            }
        }
        Err(e) => {
            warn!(
                "Failed to get MV for default variant of schema {}: {}",
                schema_id, e
            );
            // Fall through to check if schema exists
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists but MV lookup failed - trigger rebuild and return 202
                    if let Err(e) = edda_client.rebuild_for_deployment().await {
                        warn!("Failed to trigger MV rebuild: {}", e);
                    }
                    let building_response = BuildingResponseV1 {
                        status: "building".to_string(),
                        message: "Schema variant data is being generated, please retry shortly"
                            .to_string(),
                        retry_after_seconds: 2,
                        estimated_completion_seconds: 10,
                    };
                    return Ok(SchemaVariantResponseV1::Building(building_response));
                }
                Ok(None) => {
                    // Schema doesn't exist - return 404
                }
                Err(_) => {
                    // Can't check DB - fall through to 404
                }
            }
        }
    }

    // Schema not found in either DAL or cached_modules
    Err(SchemaError::SchemaNotFound(schema_id))
}
