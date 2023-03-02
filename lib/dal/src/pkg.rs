use std::path::PathBuf;

use si_pkg::{
    PkgSpec, PropSpec, PropSpecKind, SchemaSpec, SchemaVariantSpec, SiPkg, SiPkgError, SiPkgProp,
    SiPkgSchema, SiPkgSchemaVariant, SpecError,
};
use thiserror::Error;

use crate::{
    component::ComponentKind,
    schema::{
        variant::definition::{hex_color_to_i64, SchemaVariantDefinitionError},
        SchemaUiMenu,
    },
    DalContext, Prop, PropError, PropId, PropKind, Schema, SchemaError, SchemaVariant,
    SchemaVariantError, SchemaVariantId, StandardModel, StandardModelError,
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
    PkgSpec(#[from] SpecError),
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

pub async fn import_pkg(ctx: &DalContext, pkg_file_path: impl Into<PathBuf>) -> PkgResult<SiPkg> {
    let pkg = SiPkg::load_from_file(pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg).await?;

    Ok(pkg)
}

// TODO(fnichol): another first-pass function with arguments. At the moment we're passing a list of
// `SchemaVariantId`s in an effort to export specific schema/variant combos but this will change in
// the future to be more encompassing. And yes, to many function args, way too many--and they're
// all `String`s
pub async fn export_pkg(
    _ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf>,
    name: impl Into<String>,
    version: impl Into<String>,
    description: Option<impl Into<String>>,
    created_by: impl Into<String>,
    _variant_ids: Vec<SchemaVariantId>,
) -> PkgResult<()> {
    let mut spec_builder = PkgSpec::builder();
    spec_builder
        .name(name)
        .version(version)
        .created_by(created_by);
    if let Some(description) = description {
        spec_builder.description(description);
    }

    // // TODO(fnichol): this is merely an example to see if a chained builder pattern works and
    // // compile--and it does!
    // //
    spec_builder
        .schema(
            SchemaSpec::builder()
                .name("Laika")
                .category("Space Dogs")
                .variant(
                    SchemaVariantSpec::builder()
                        .name("v0")
                        .color("4695E7")
                        .prop(
                            PropSpec::builder()
                                .name("age")
                                .kind(PropSpecKind::Number)
                                .build()?,
                        )
                        .prop(
                            PropSpec::builder()
                                .name("praises")
                                .kind(PropSpecKind::Array)
                                .type_prop(
                                    PropSpec::builder()
                                        .name("praiseString")
                                        .kind(PropSpecKind::String)
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let spec = spec_builder.build()?;
    let pkg = SiPkg::load_from_spec(spec)?;
    pkg.write_to_file(pkg_file_path).await?;

    Ok(())
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
