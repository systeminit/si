use dal::pkg::{import_pkg_from_pkg, ImportOptions};
use dal::{prop::PropPath, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind, SchemaId};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, PkgSpec, PropSpec, SchemaSpec, SchemaVariantSpec,
    SchemaVariantSpecData, SiPkg,
};
use si_pkg::{SchemaSpecData, SocketSpec, SocketSpecData, SocketSpecKind};

use crate::test_exclusive_schemas::{
    build_asset_func, create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

const CATEGORY: &str = "pirate";

pub(crate) async fn migrate_test_exclusive_schema_pirate(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "pirate";

    builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldPirateAsset";
    let authoring_schema_func = build_asset_func(fn_name)?;

    let schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(CATEGORY)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ff00ff")
                        .func_unique_id(&authoring_schema_func.unique_id)
                        .component_type(ComponentType::Component)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("name")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::Prop)
                                .name("identity")
                                .prop_path(PropPath::new(["root", "si", "name"]))
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("working_eyes")
                        .kind(PropKind::Integer)
                        .validation_format(r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":0}},{"name":"max","args":{"limit":2}}]}"#)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("parrot_names")
                        .kind(PropKind::Array)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .type_prop(
                            PropSpec::builder()
                                .kind(PropKind::String)
                                .name("parrot_name")
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("parrot_names")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("treasure")
                        .kind(PropKind::Map)
                        .type_prop(
                            PropSpec::builder()
                                .kind(PropKind::String)
                                .name("location")
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("parrot_names")
                        .data(
                            SocketSpecData::builder()
                                .name("parrot_names")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "parrot_names",
                                ])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let spec = builder
        .func(identity_func_spec)
        .func(authoring_schema_func)
        .schema(schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_pet_shop(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "pet_shop";

    builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldPetShopAsset";
    let authoring_schema_func = build_asset_func(fn_name)?;

    let schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(CATEGORY)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ff00ff")
                        .func_unique_id(&authoring_schema_func.unique_id)
                        .component_type(ComponentType::Component)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("parrot_names")
                        .kind(PropKind::Array)
                        .type_prop(
                            PropSpec::builder()
                                .kind(PropKind::String)
                                .name("parrot_name")
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("parrot_names")
                        .data(
                            SocketSpecData::builder()
                                .name("parrot_names")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "parrot_names",
                                ])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "parrot_names"]))
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let spec = builder
        .func(identity_func_spec)
        .func(authoring_schema_func)
        .schema(schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
