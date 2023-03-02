use std::path::Path;

use si_pkg::{SiPkg, SiPkgError, SiPkgProp, SiPkgSchema, SiPkgSchemaVariant};
use thiserror::Error;

use crate::{
    component::ComponentKind,
    schema::{
        variant::definition::{hex_color_to_i64, SchemaVariantDefinitionError},
        SchemaUiMenu,
    },
    DalContext, Prop, PropError, PropId, PropKind, Schema, SchemaError, SchemaVariant,
    SchemaVariantError, StandardModel, StandardModelError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type PkgResult<T> = Result<T, PkgError>;

pub async fn import_pkg_from_pkg(ctx: &DalContext, pkg: &SiPkg) -> PkgResult<()> {
    for schema_spec in pkg.schemas()? {
        create_schema(ctx, schema_spec).await?;
    }

    Ok(())
}

pub async fn import_pkg(ctx: &DalContext, tar_path: &Path) -> PkgResult<SiPkg> {
    let pkg = SiPkg::load_from_file(tar_path).await?;

    import_pkg_from_pkg(ctx, &pkg).await?;

    Ok(pkg)
}

async fn create_schema(ctx: &DalContext, schema_spec: SiPkgSchema<'_>) -> PkgResult<()> {
    let mut schema = Schema::new(ctx, schema_spec.name(), &ComponentKind::Standard).await?;
    let ui_menu = SchemaUiMenu::new(ctx, schema_spec.name(), schema_spec.category()).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    for variant_spec in schema_spec.variants()? {
        create_schema_variant(ctx, &mut schema, variant_spec).await?;
    }

    Ok(())
}

async fn create_schema_variant(
    ctx: &DalContext,
    schema: &mut Schema,
    variant_spec: SiPkgSchemaVariant<'_>,
) -> PkgResult<()> {
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
