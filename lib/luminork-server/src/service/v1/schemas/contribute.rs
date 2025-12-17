use axum::{
    Json,
    extract::Path,
};
use chrono::Utc;
use dal::{
    SchemaVariant,
    module::Module,
};
use module_index_client::ModuleIndexClient;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
    request::RawAccessToken,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
};

use super::{
    SchemaError,
    SchemaResult,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/contribute",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
    ),
    tag = "schemas",
    summary = "Contribute the default variant of a schema to the module index",
    responses(
        (status = 200, description = "Schema default variant contributed successfully"),
        (status = 400, description = "Bad request - Cannot contribute on head change set or default variant not locked or schema installed from upstream"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn contribute(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    RawAccessToken(raw_access_token): RawAccessToken,
    tracker: PosthogEventTracker,
    Path(super::SchemaV1RequestPath { schema_id }): Path<super::SchemaV1RequestPath>,
) -> SchemaResult<Json<serde_json::Value>> {
    // Check if the change set is HEAD - we ONLY allow contributions on HEAD
    let head_change_set_id = ctx.get_workspace_default_change_set_id().await?;
    if ctx.change_set_id() != head_change_set_id {
        return Err(SchemaError::ContributionsMustBeMadeFromHead);
    }

    // Get the default variant for the schema
    let schema_variant_id = dal::Schema::default_variant_id(ctx, schema_id).await?;
    let variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;

    // Check if the variant is NOT locked - we only allow contributions on locked variants
    if !variant.is_locked() {
        return Err(SchemaError::ContributeUnlockedVariant(schema_variant_id));
    }

    // We have 3 scenarios in which we accept contributions:
    // 1. A user downloads a builtin and immediately contributes it back
    //   - This is not a usual scenario and something we will not accept anyway
    // 2. A user creates a modification of a builtin and contributes it back
    //   - This is a more realistic scenario but all of our builtins are managed via clover
    //   - So this isn't something we will support and we will document that for it not being
    //   - the right way to change a builtin
    // 3. A user generates a NEW asset and wants to contribute it back to us
    //
    // It is on US as System Initiative to be able to effectively understand what differences
    // there are in the contribution. Stopping someone from contribute stops them from using
    // our tool and we should encourage their usage!

    let module_index_url = ctx
        .module_index_url()
        .ok_or(SchemaError::ModuleIndexNotConfigured)?;
    let index_client = ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;

    let version = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let formatted_name = format!("{} {}", variant.display_name(), version);
    let (
        name,
        version,
        based_on_hash,
        schema_id_from_module,
        payload,
        created_by_name,
        created_by_email,
        schema_variant_version,
    ) = Module::prepare_contribution(ctx, formatted_name, &version, schema_variant_id, false)
        .await?;

    let response = index_client
        .upload_module(
            name.as_str(),
            version.as_str(),
            based_on_hash.clone(),
            schema_id_from_module.map(|id| id.to_string()),
            payload.clone(),
            Some(schema_variant_id.to_string()),
            Some(schema_variant_version.clone()),
            Some(false),
        )
        .await?;

    ctx.write_audit_log(
        AuditLogKind::ContributeModule {
            version: version.clone(),
            schema_id: schema_id_from_module,
            schema_variant_id: schema_variant_id.into(),
            schema_variant_version: Some(schema_variant_version.clone()),
        },
        name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_contribute_schema",
        json!({
            "pkg_name": name,
            "pkg_version": version,
            "based_on_hash": based_on_hash,
            "pkg_created_by_name": created_by_name,
            "pkg_created_by_email": created_by_email,
            "schema_variant_id": schema_variant_id,
            "schema_id": schema_id,
            "pkg_hash": response.latest_hash,
        }),
    );

    ctx.commit().await?;

    Ok(Json(json!({
        "success": true,
        "hash": response.latest_hash,
    })))
}
