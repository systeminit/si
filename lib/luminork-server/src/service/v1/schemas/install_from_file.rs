use axum::{
    Json,
    extract::Multipart,
};
use dal::{
    Schema,
    SchemaVariant,
    SchemaVariantId,
    cached_module::CachedModule,
    pkg::{
        ImportOptions,
        import_funcs_for_module_update,
        import_pkg_from_pkg,
        import_schema_variant,
    },
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use si_pkg::{
    PkgSpec,
    SiPkg,
};
use telemetry::prelude::*;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    SchemaError,
    SchemaResult,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InstallFromFileV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub schema_id: si_events::SchemaId,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub schema_variant_id: si_events::SchemaVariantId,
    #[schema(example = "AWS::EC2::Instance")]
    pub schema_name: String,
    #[schema(example = "EC2 Instance")]
    pub display_name: String,
    #[schema(example = "AWS::EC2")]
    pub category: String,
}

/// Install a schema from a PkgSpec file
///
/// Accepts a multipart form with a `pkg_spec` field containing the JSON PkgSpec.
/// If the schema already exists, it will be upgraded with the new variant.
#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/install_from_file",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "schemas",
    summary = "Install a schema from a PkgSpec file",
    responses(
        (status = 200, description = "Schema installed successfully", body = InstallFromFileV1Response),
        (status = 400, description = "Bad request - Invalid or missing pkg_spec", body = crate::service::v1::common::ApiError),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid PkgSpec data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn install_from_file(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    mut multipart: Multipart,
) -> SchemaResult<Json<InstallFromFileV1Response>> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    // Extract the pkg_spec field from multipart form
    let mut maybe_module_json = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("pkg_spec") => {
                maybe_module_json = Some(field.bytes().await?);
            }
            _ => debug!("Unknown multipart form field on module install, skipping..."),
        }
    }

    let Some(module_bytes) = maybe_module_json else {
        return Err(SchemaError::PkgFileError("Missing pkg_spec field"));
    };

    let module_string = String::from_utf8_lossy(&module_bytes);
    let spec: PkgSpec = serde_json::from_str(&module_string)?;
    let pkg = SiPkg::load_from_spec(spec)?;

    // Validate that the package has exactly one schema
    let schemas = pkg.schemas()?;
    if schemas.len() != 1 {
        return Err(SchemaError::PkgFileError(
            "Pkg must have exactly one schema",
        ));
    }

    let schema_spec = schemas
        .first()
        .ok_or(SchemaError::PkgFileError("Pkg has no schemas"))?;
    let schema_name = schema_spec.name();

    // Check if schema exists in change set or module cache
    let schema_exists_in_cache = CachedModule::find_latest_for_schema_name(ctx, schema_name)
        .await?
        .is_some();

    let variant_ids = match Schema::get_by_name_opt(ctx, schema_name).await? {
        Some(existing_schema) => {
            upgrade_schema_from_uploaded_file(ctx, &pkg, existing_schema).await?
        }
        None if schema_exists_in_cache => {
            let installed_schema = Schema::get_or_install_by_name(ctx, schema_name).await?;
            upgrade_schema_from_uploaded_file(ctx, &pkg, installed_schema).await?
        }
        None => {
            let (_, variant_ids, _) = import_pkg_from_pkg(
                ctx,
                &pkg,
                Some(ImportOptions {
                    schema_id: None,
                    past_module_hashes: None,
                    ..Default::default()
                }),
            )
            .await
            .map_err(SchemaError::Pkg)?;
            variant_ids
        }
    };

    let schema_variant_id = variant_ids
        .first()
        .ok_or(SchemaError::PkgFileError("Pkg has no variants"))?;

    let variant = SchemaVariant::get_by_id(ctx, *schema_variant_id).await?;
    let schema = variant.schema(ctx).await?;

    tracker.track(
        ctx,
        "api_install_from_file",
        json!({
            "schema_id": schema.id(),
            "schema_variant_id": variant.id(),
            "display_name": variant.display_name(),
            "category": variant.category(),
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::CreateSchemaVariant {
            schema_id: schema.id(),
            schema_variant_id: variant.id(),
        },
        variant.display_name().to_string(),
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(InstallFromFileV1Response {
        schema_id: schema.id(),
        schema_variant_id: variant.id(),
        schema_name: schema.name.clone(),
        display_name: variant.display_name().to_string(),
        category: variant.category().to_string(),
    }))
}

async fn upgrade_schema_from_uploaded_file(
    ctx: &dal::DalContext,
    pkg: &SiPkg,
    schema: Schema,
) -> SchemaResult<Vec<SchemaVariantId>> {
    // Import and update funcs from uploaded pkg
    let mut thing_map = import_funcs_for_module_update(ctx, pkg.funcs()?)
        .await
        .map_err(SchemaError::Pkg)?;

    // Get specs from uploaded pkg
    let pkg_schemas = pkg.schemas()?;
    let schema_spec = pkg_schemas
        .first()
        .ok_or(SchemaError::PkgFileError("Pkg has no schemas"))?;
    let variants = schema_spec.variants()?;
    let variant_spec = variants
        .first()
        .ok_or(SchemaError::PkgFileError("Schema has no variants"))?;

    // Create new variant from uploaded pkg
    let new_variant = import_schema_variant(
        ctx,
        &schema,
        schema_spec.clone(),
        variant_spec,
        None,
        &mut thing_map,
        None,
    )
    .await
    .map_err(SchemaError::Pkg)?;

    // Set as new default
    schema
        .set_default_variant_id(ctx, new_variant.id())
        .await
        .map_err(|e| SchemaError::Pkg(e.into()))?;

    Ok(vec![new_variant.id()])
}
