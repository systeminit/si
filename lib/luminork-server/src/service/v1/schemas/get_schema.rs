use axum::extract::Path;
use dal::{
    Schema,
    SchemaVariant,
    cached_module::CachedModule,
};
use itertools::Itertools;
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_schema::CachedSchema,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    BuildingResponseV1,
    GetSchemaV1Response,
    SchemaError,
    SchemaResponseV1,
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
        (status = 202, description = "Schema data is being generated from cached modules", body = BuildingResponseV1),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(frigg): FriggStore,
    _edda_client: EddaClient,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<SchemaResponseV1> {
    // Try DAL lookup first (installed schema)
    if let Ok(Some(schema)) = Schema::get_by_id_opt(ctx, schema_id).await {
        let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
        let variants = SchemaVariant::list_for_schema(ctx, schema_id).await?;

        // Check if an upgrade is available
        let upgrade_available = super::check_schema_upgrade_available(ctx, schema_id).await?;

        tracker.track(
            ctx,
            "api_get_schema",
            json!({
                "schema_id": schema_id,
                "schema_name": schema.name,
                "default_variant_id": default_variant_id,
                "source": "dal"
            }),
        );

        return Ok(SchemaResponseV1::Success(GetSchemaV1Response {
            schema_id,
            name: schema.name,
            default_variant_id,
            variant_ids: variants.into_iter().map(|v| v.id).collect_vec(),
            upgrade_available,
        }));
    }

    // Fall back to MV lookup for uninstalled schemas using CachedSchema
    match frigg
        .get_current_deployment_object(ReferenceKind::CachedSchema.into(), &schema_id.to_string())
        .await
    {
        Ok(Some(obj)) => {
            if let Ok(cached_schema) = serde_json::from_value::<CachedSchema>(obj.data) {
                tracker.track(
                    ctx,
                    "api_get_schema",
                    json!({
                        "schema_id": schema_id,
                        "schema_name": cached_schema.name,
                        "default_variant_id": cached_schema.default_variant_id,
                        "source": "materialized_view"
                    }),
                );

                return Ok(SchemaResponseV1::Success(GetSchemaV1Response {
                    schema_id,
                    name: cached_schema.name,
                    default_variant_id: cached_schema.default_variant_id,
                    variant_ids: cached_schema.variant_ids,
                    upgrade_available: None, // Not installed, so no upgrade check possible
                }));
            }
        }
        Ok(None) => {
            // CachedSchema not found - check if schema exists in cached_modules DB table
            match CachedModule::find_latest_for_schema_id(ctx, schema_id).await {
                Ok(Some(_)) => {
                    // Schema exists in cached_modules but CachedSchema MV not built yet - trigger rebuild and return 202
                    //
                    // Until the performance issues in building the deployment-level MVs are fixed,
                    // this is only going to be deployed through the manual module sync process.
                    //
                    // if let Err(e) = edda_client.rebuild_for_deployment().await {
                    //     warn!("Failed to trigger MV rebuild: {}", e);
                    // }

                    return Ok(SchemaResponseV1::Building(Box::new(
                        BuildingResponseV1::new_and_increment_counter_for_schema_cached_modules(),
                    )));
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
            warn!("Failed to get MV for schema {}: {}", schema_id, e);
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

                    return Ok(SchemaResponseV1::Building(Box::new(
                        BuildingResponseV1::new_and_increment_counter_for_schema_cached_modules(),
                    )));
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
