use axum::extract::Multipart;
use dal::{
    ChangeSet,
    DalContext,
    Func,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    WsEvent,
    cached_module::CachedModule,
    pkg::{
        ImportOptions,
        import_funcs_for_module_update,
        import_pkg_from_pkg,
        import_schema_variant,
    },
};
use si_frontend_types::SchemaVariant as FrontendVariant;
use si_pkg::{
    PkgSpec,
    SiPkg,
};
use telemetry::prelude::*;

use crate::{
    extract::change_set::ChangeSetDalContext,
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::module::ModulesAPIError,
    },
};

pub async fn install_module_from_file(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    mut multipart: Multipart,
) -> Result<ForceChangeSetResponse<Vec<FrontendVariant>>, ModulesAPIError> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

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
        return Err(ModulesAPIError::PkgFileError("Missing file"));
    };

    let module_string = String::from_utf8_lossy(&module_bytes);

    let spec: PkgSpec = serde_json::from_str(&module_string)?;
    let pkg = SiPkg::load_from_spec(spec)?;

    let mut variants = Vec::new();

    // After validating that we can install the modules, get on with it.
    let schemas = pkg.schemas()?;
    if schemas.len() != 1 {
        return Err(ModulesAPIError::PkgFileError(
            "Pkg has more than one schema",
        ));
    }

    // Extract schema name and check if it already exists
    let schema_spec = schemas
        .first()
        .ok_or(ModulesAPIError::PkgFileError("Pkg has no schemas"))?;
    let schema_name = schema_spec.name();

    // Check if schema exists in change set or module cache
    let schema_exists_in_cache = CachedModule::find_latest_for_schema_name(ctx, schema_name)
        .await?
        .is_some();

    let variant_ids = match Schema::get_by_name_opt(ctx, schema_name)
        .await
        .map_err(|e| ModulesAPIError::Pkg(e.into()))?
    {
        Some(existing_schema) => {
            upgrade_schema_from_uploaded_file(ctx, &pkg, existing_schema).await?
        }
        None if schema_exists_in_cache => {
            let installed_schema = Schema::get_or_install_by_name(ctx, schema_name)
                .await
                .map_err(|e| ModulesAPIError::Pkg(e.into()))?;
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
            .await?;
            variant_ids
        }
    };

    if let Some(schema_variant_id) = variant_ids.first() {
        let variant = SchemaVariant::get_by_id(ctx, *schema_variant_id).await?;
        let schema_id = variant.schema(ctx).await?.id();
        let front_end_variant = variant.into_frontend_type(ctx, schema_id).await?;
        WsEvent::module_imported(ctx, vec![front_end_variant.clone()])
            .await?
            .publish_on_commit(ctx)
            .await?;
        for func_id in front_end_variant.func_ids.iter() {
            let func = Func::get_by_id(ctx, *func_id).await?;
            let front_end_func = func.into_frontend_type(ctx).await?;
            WsEvent::func_updated(ctx, front_end_func, None)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        variants.push(front_end_variant);
    } else {
        return Err(ModulesAPIError::PkgFileError("Pkg has no variants"));
    };

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, variants))
}

async fn upgrade_schema_from_uploaded_file(
    ctx: &DalContext,
    pkg: &SiPkg,
    schema: Schema,
) -> Result<Vec<SchemaVariantId>, ModulesAPIError> {
    // Import and update funcs from uploaded pkg using UpdateExisting mode
    let mut thing_map = import_funcs_for_module_update(ctx, pkg.funcs()?)
        .await
        .map_err(ModulesAPIError::Pkg)?;

    // Get specs from uploaded pkg
    let pkg_schemas = pkg.schemas()?;
    let schema_spec = pkg_schemas
        .first()
        .ok_or(ModulesAPIError::PkgFileError("Pkg has no schemas"))?;
    let variants = schema_spec.variants()?;
    let variant_spec = variants
        .first()
        .ok_or(ModulesAPIError::PkgFileError("Schema has no variants"))?;

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
    .map_err(ModulesAPIError::Pkg)?;

    // Set as new default
    schema
        .set_default_variant_id(ctx, new_variant.id())
        .await
        .map_err(|e| ModulesAPIError::Pkg(e.into()))?;

    Ok(vec![new_variant.id()])
}
