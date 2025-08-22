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
    cached_schema_variant::CachedSchemaVariant,
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
    _edda_client: EddaClient,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
) -> SchemaResult<SchemaVariantResponseV1> {
    // Try DAL lookup first (installed schema variants) - but only if schema exists in DAL
    if let Ok(schema_variants) = Schema::list_schema_variant_ids(ctx, schema_id).await {
        if schema_variants.contains(&schema_variant_id) {
            if let Ok(Some(variant)) = SchemaVariant::get_by_id_opt(ctx, schema_variant_id).await {
                let variant_func_ids: Vec<_> = SchemaVariant::all_func_ids(ctx, schema_variant_id)
                    .await?
                    .into_iter()
                    .collect();

                let domain = Prop::find_prop_by_path(
                    ctx,
                    schema_variant_id,
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
                    "api_get_variant",
                    json!({
                        "schema_id": schema_id,
                        "schema_variant_id": schema_variant_id,
                        "schema_variant_name": variant.display_name(),
                        "schema_variant_category": variant.category(),
                        "is_locked": variant.is_locked(),
                        "source": "dal"
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(
                    GetSchemaVariantV1Response {
                        variant_id: schema_variant_id,
                        display_name: variant.display_name().into(),
                        category: variant.category().into(),
                        color: variant.color().into(),
                        is_locked: variant.is_locked(),
                        description: variant.description(),
                        link: variant.link(),
                        asset_func_id: variant.asset_func_id_or_error()?,
                        variant_func_ids,
                        is_default_variant: SchemaVariant::is_default_by_id(ctx, schema_variant_id)
                            .await?,
                        domain_props: Some(domain_prop_schema),
                    },
                )));
            }
        }
    }

    // Fall back to MV lookup for uninstalled schema variants
    match frigg
        .get_current_deployment_object(
            ReferenceKind::CachedSchemaVariant,
            &schema_variant_id.to_string(),
        )
        .await
    {
        Ok(Some(obj)) => {
            if let Ok(cached_variant) = serde_json::from_value::<CachedSchemaVariant>(obj.data) {
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
                    is_default_variant: cached_variant.is_default_variant,
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
                        "is_locked": cached_variant.is_locked,
                        "source": "materialized_view"
                    }),
                );

                return Ok(SchemaVariantResponseV1::Success(Box::new(response)));
            }
        }
        Ok(None) => {
            // CachedSchemaVariant not found - check if schema exists in cached_modules DB table
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists in cached_modules but CachedSchemaVariant MV not built yet - trigger rebuild and return 202
                    //
                    // Until the performance issues in building the deployment-level MVs are fixed,
                    // this is only going to be deployed through the manual module sync process.
                    //
                    // if let Err(e) = edda_client.rebuild_for_deployment().await {
                    //     warn!("Failed to trigger MV rebuild: {}", e);
                    // }

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
        Err(e) => {
            warn!("Failed to get MV for variant {}: {}", schema_variant_id, e);
            // Fall back to check if schema exists in cached_modules
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists but MV lookup failed - trigger rebuild and return building response
                    //
                    // Until the performance issues in building the deployment-level MVs are fixed,
                    // this is only going to be deployed through the manual module sync process.
                    //
                    // if let Err(e) = edda_client.rebuild_for_deployment().await {
                    //     warn!("Failed to trigger MV rebuild: {}", e);
                    // }

                    return Ok(SchemaVariantResponseV1::Building(BuildingResponseV1 {
                        status: "building".to_string(),
                        message: "Schema variant data is being generated from cached modules, please retry shortly".to_string(),
                        retry_after_seconds: 2,
                        estimated_completion_seconds: 10,
                    }));
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

    // Schema variant not found in either DAL or MV
    Err(SchemaError::SchemaVariantNotFound(schema_variant_id))
}
