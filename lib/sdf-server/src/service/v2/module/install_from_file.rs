use axum::extract::Multipart;
use dal::{
    ChangeSet,
    Func,
    SchemaVariant,
    WsEvent,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
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
    if pkg.schemas()?.len() != 1 {
        return Err(ModulesAPIError::PkgFileError(
            "Pkg has more than one schema",
        ));
    }

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
