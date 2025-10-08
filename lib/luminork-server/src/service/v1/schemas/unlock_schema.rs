use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    schema::variant::authoring::VariantAuthoringClient,
    slow_rt,
};
use sdf_extract::{
    FriggStore,
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use si_frontend_mv_types::reference::ReferenceKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
};
use crate::service::v1::{
    GetSchemaVariantV1Response,
    schemas::SchemaVariantFunc,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/unlock",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
    ),
    tag = "schemas",
    summary = "Unlocks a schema - if there's already an unlocked variant, then we return that",
    responses(
        (status = 200, description = "Schema unlocked successfully", body = UnlockedSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn unlock_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(frigg): FriggStore,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
) -> SchemaResult<Json<UnlockedSchemaV1Response>> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema = if let Some(installed_schema) = Schema::get_by_id_opt(ctx, schema_id).await? {
        installed_schema
    } else {
        // Let's try and install the schema and ensure that we installed it correctly
        Schema::get_or_install_default_variant(ctx, schema_id).await?;
        Schema::get_by_id_opt(ctx, schema_id)
            .await?
            .ok_or(SchemaError::SchemaNotFound(schema_id))?
    };

    let default_variant_id = Schema::default_variant_id(ctx, schema_id).await?;
    let variants = SchemaVariant::list_for_schema(ctx, schema_id).await?;

    let unlocked_variant_id = match variants.iter().find(|v| !v.is_locked()) {
        Some(v) => v.id(), // already have one so we will return that unlocked variant id
        None => {
            // Otherwise lets create one!
            let unlocked =
                VariantAuthoringClient::create_unlocked_variant_copy(ctx, default_variant_id)
                    .await?;
            ctx.write_audit_log(
                AuditLogKind::UnlockSchemaVariant {
                    schema_variant_id: unlocked.id(),
                    schema_variant_display_name: unlocked.display_name().to_owned(),
                },
                schema.name().to_owned(),
            )
            .await?;
            unlocked.id()
        }
    };

    tracker.track(
        ctx,
        "api_unlock_schema",
        json!({
            "schema_id": schema.id(),
            "unlocked_variant_id": unlocked_variant_id,
        }),
    );

    ctx.commit().await?;

    let luminork_variant =
        slow_rt::spawn(dal_materialized_views::luminork::schema::variant::assemble(
            ctx.clone(),
            unlocked_variant_id,
        ))?
        .await??;

    let variant_funcs: Vec<SchemaVariantFunc> = luminork_variant
        .variant_funcs
        .into_iter()
        .map(SchemaVariantFunc::from)
        .collect();

    // We know it is a builtin if we find a CachedSchema for its schema id
    // The only cached schemas we currently build are builtins - if that changes, this logic will need to change!
    let installed_from_upstream = (frigg
        .get_current_deployment_object(ReferenceKind::CachedSchema.into(), &schema_id.to_string())
        .await?)
        .is_some();

    let unlocked_variant = GetSchemaVariantV1Response {
        variant_id: luminork_variant.variant_id,
        display_name: luminork_variant.display_name,
        category: luminork_variant.category,
        color: luminork_variant.color,
        is_locked: luminork_variant.is_locked,
        installed_from_upstream,
        description: luminork_variant.description,
        link: luminork_variant.link,
        asset_func_id: luminork_variant.asset_func_id,
        variant_func_ids: luminork_variant.variant_func_ids,
        variant_funcs,
        is_default_variant: luminork_variant.is_default_variant,
        domain_props: luminork_variant.domain_props.map(Into::into),
    };

    Ok(Json(UnlockedSchemaV1Response {
        schema_id,
        unlocked_variant_id,
        unlocked_variant,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnlockedSchemaV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub schema_id: SchemaId,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub unlocked_variant_id: SchemaVariantId,
    pub unlocked_variant: GetSchemaVariantV1Response,
}
