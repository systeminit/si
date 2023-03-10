use std::path::PathBuf;

use si_pkg::{SiPkg, SiPkgError, SiPkgFunc, SiPkgProp, SiPkgSchema, SiPkgSchemaVariant};

use crate::{
    component::ComponentKind,
    installed_pkg::{
        InstalledPkg, InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped,
        InstalledPkgId,
    },
    schema::{variant::definition::hex_color_to_i64, SchemaUiMenu},
    DalContext, Func, Prop, PropId, PropKind, Schema, SchemaVariant, StandardModel,
};

use super::{PkgError, PkgResult};

pub async fn import_pkg_from_pkg(ctx: &DalContext, pkg: &SiPkg, file_name: &str) -> PkgResult<()> {
    // We have to write the installed_pkg row first, so that we have an id, and rely on transaction
    // semantics to remove the row if anything in the installation process fails
    let root_hash = pkg.hash()?.to_string();

    if !InstalledPkg::find_by_attr(ctx, "root_hash", &root_hash)
        .await?
        .is_empty()
    {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let installed_pkg = InstalledPkg::new(ctx, &file_name, pkg.hash()?.to_string()).await?;

    for func_spec in pkg.funcs()? {
        create_func(ctx, func_spec, *installed_pkg.id()).await?;
    }

    // TODO: gather up a record of what wasn't installed and why (the id of the package that
    // already contained the schema or variant)
    for schema_spec in pkg.schemas()? {
        create_schema(ctx, schema_spec, *installed_pkg.id()).await?;
    }

    Ok(())
}

pub async fn import_pkg(
    ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf> + Clone,
) -> PkgResult<SiPkg> {
    let pkg_file_path_str = Into::<PathBuf>::into(pkg_file_path.clone())
        .to_string_lossy()
        .to_string();

    let pkg = SiPkg::load_from_file(pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, &pkg_file_path_str).await?;

    Ok(pkg)
}

async fn create_func(
    ctx: &DalContext,
    func_spec: SiPkgFunc<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<()> {
    let hash = func_spec.hash().to_string();
    let existing_func =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Func, &hash)
            .await?
            .pop();

    let func_id = match existing_func {
        Some(installed_func_record) => match installed_func_record.as_installed_func()? {
            InstalledPkgAssetTyped::Func { id, .. } => id,
            _ => unreachable!(),
        },
        None => {
            // How to handle name conflicts?
            let mut func = Func::new(
                ctx,
                func_spec.name(),
                func_spec.backend_kind().into(),
                func_spec.response_type().into(),
            )
            .await?;

            func.set_display_name(ctx, func_spec.display_name()).await?;
            func.set_code_base64(ctx, Some(func_spec.code_base64()))
                .await?;
            func.set_description(ctx, func_spec.description()).await?;
            func.set_handler(ctx, Some(func_spec.handler())).await?;
            func.set_hidden(ctx, func.hidden()).await?;
            func.set_link(ctx, func_spec.link().map(|l| l.to_string()))
                .await?;

            *func.id()
        }
    };

    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_func(func_id, installed_pkg_id, hash),
    )
    .await?;

    Ok(())
}

async fn create_schema(
    ctx: &DalContext,
    schema_spec: SiPkgSchema<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<()> {
    let hash = schema_spec.hash().to_string();
    let existing_schema =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Schema, &hash)
            .await?
            .pop();

    let mut schema = match existing_schema {
        None => {
            let schema = Schema::new(ctx, schema_spec.name(), &ComponentKind::Standard).await?;
            let ui_menu =
                SchemaUiMenu::new(ctx, schema_spec.name(), schema_spec.category()).await?;
            ui_menu.set_schema(ctx, schema.id()).await?;

            schema
        }
        Some(installed_schema_record) => match installed_schema_record.as_installed_schema()? {
            InstalledPkgAssetTyped::Schema { id, .. } => match Schema::get_by_id(ctx, &id).await? {
                Some(schema) => schema,
                None => return Err(PkgError::InstalledSchemaMissing(id)),
            },
            _ => unreachable!(),
        },
    };

    // Even if the asset is already installed, we write a record of the asset installation so that
    // we can track the installed packages that share schemas.
    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema(*schema.id(), installed_pkg_id, hash),
    )
    .await?;

    for variant_spec in schema_spec.variants()? {
        create_schema_variant(ctx, &mut schema, variant_spec, installed_pkg_id).await?;
    }

    Ok(())
}

async fn create_schema_variant(
    ctx: &DalContext,
    schema: &mut Schema,
    variant_spec: SiPkgSchemaVariant<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<()> {
    let hash = variant_spec.hash().to_string();
    let existing_schema_variant =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::SchemaVariant, &hash)
            .await?
            .pop();

    let variant_id = match existing_schema_variant {
        Some(installed_sv_record) => match installed_sv_record.as_installed_schema_variant()? {
            InstalledPkgAssetTyped::SchemaVariant { id, .. } => id,
            _ => unreachable!(),
        },
        None => {
            let (mut schema_variant, root_prop) =
                SchemaVariant::new(ctx, *schema.id(), variant_spec.name()).await?;

            schema
                .set_default_schema_variant_id(ctx, Some(schema_variant.id()))
                .await?;

            let color = match variant_spec.color() {
                Some(color_str) => Some(hex_color_to_i64(color_str)?),
                None => None,
            };
            schema_variant.set_color(ctx, color).await?;

            let domain_prop_id = root_prop.domain_prop_id;
            variant_spec
                .visit_prop_tree(create_prop, Some(domain_prop_id), ctx)
                .await?;

            schema_variant.finalize(ctx, None).await?;
            *schema_variant.id()
        }
    };

    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema_variant(variant_id, installed_pkg_id, hash),
    )
    .await?;

    Ok(())
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_id: Option<PropId>,
    ctx: &DalContext,
) -> Result<Option<PropId>, SiPkgError> {
    let prop = Prop::new(
        ctx,
        spec.name(),
        match spec {
            SiPkgProp::String { .. } => PropKind::String,
            SiPkgProp::Number { .. } => PropKind::Integer,
            SiPkgProp::Boolean { .. } => PropKind::Boolean,
            SiPkgProp::Map { .. } => PropKind::Map,
            SiPkgProp::Array { .. } => PropKind::Array,
            SiPkgProp::Object { .. } => PropKind::Object,
        },
        None,
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    if let Some(parent_prop_id) = parent_prop_id {
        prop.set_parent_prop(ctx, parent_prop_id)
            .await
            .map_err(SiPkgError::visit_prop)?;
    }

    Ok(Some(*prop.id()))
}
