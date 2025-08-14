use axum::extract::Path;
use dal::{
    Prop,
    Schema,
    SchemaVariant,
    schema::variant::root_prop::RootPropChild,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_schema::CachedSchema,
    cached_schema_variant::CachedSchemaVariant,
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
            let domain_prop_schema = build_prop_schema_tree(ctx, domain.id).await?;

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

            return Ok(SchemaVariantResponseV1::Success(
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
            ));
        }
    }

    // Fall back to MV lookup for uninstalled schemas
    // For default variant, we need to find the schema in cached modules first
    match frigg
        .get_current_deployment_objects_by_kind(ReferenceKind::CachedSchema)
        .await
    {
        Ok(cached_schemas) => {
            for cached_schema_obj in cached_schemas {
                if let Ok(cached_schema) =
                    serde_json::from_value::<CachedSchema>(cached_schema_obj.data)
                {
                    if cached_schema.id == schema_id {
                        // Found the schema, now get its default variant
                        let default_variant_id = cached_schema.default_variant_id;
                        match frigg
                            .get_current_deployment_object(
                                ReferenceKind::CachedSchemaVariant,
                                &default_variant_id.to_string(),
                            )
                            .await
                        {
                            Ok(Some(obj)) => {
                                if let Ok(cached_variant) =
                                    serde_json::from_value::<CachedSchemaVariant>(obj.data)
                                {
                                    // Build domain_props using current DAL approach since MV excludes them
                                    let domain = Prop::find_prop_by_path(
                                        ctx,
                                        default_variant_id,
                                        &RootPropChild::Domain.prop_path(),
                                    )
                                    .await
                                    .map_err(Box::new)?;
                                    let domain_prop_schema =
                                        build_prop_schema_tree(ctx, domain.id).await?;

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
                                        is_default_variant: true, // Always true for default variant endpoint
                                        domain_props: Some(domain_prop_schema),
                                    };

                                    tracker.track(
                                        ctx,
                                        "api_get_default_variant",
                                        json!({
                                            "schema_id": schema_id,
                                            "schema_variant_id": default_variant_id,
                                            "schema_variant_name": display_name_for_log,
                                            "schema_variant_category": category_for_log,
                                            "is_locked": cached_variant.is_locked,
                                            "source": "materialized_view"
                                        }),
                                    );

                                    return Ok(SchemaVariantResponseV1::Success(response));
                                }
                            }
                            Ok(None) => {
                                // MV not found, trigger rebuild and return 202
                                if let Err(e) = edda_client.rebuild_for_deployment().await {
                                    tracing::warn!("Failed to trigger MV rebuild: {}", e);
                                }
                                let building_response = BuildingResponseV1 {
                                        status: "building".to_string(),
                                        message: "Schema variant data is being generated, please retry shortly".to_string(),
                                        retry_after_seconds: 2,
                                        estimated_completion_seconds: 10,
                                    };
                                return Ok(SchemaVariantResponseV1::Building(Box::new(
                                    building_response,
                                )));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to get MV for default variant {}: {}",
                                    default_variant_id,
                                    e
                                );
                            }
                        }
                        break;
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get cached schemas: {}", e);
        }
    }

    // Schema not found in either DAL or MV
    Err(SchemaError::SchemaNotFound(schema_id))
}
