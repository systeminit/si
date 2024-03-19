use crate::schemas::schema_helpers::{build_asset_func, create_identity_func};
use dal::pkg::import_pkg_from_pkg;
use dal::{pkg, prop::PropPath, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind};
use si_pkg::SchemaSpecData;
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, PkgSpec, PropSpec, SchemaSpec, SchemaVariantSpec,
    SchemaVariantSpecData, SiPkg,
};

pub async fn migrate_test_exclusive_schema_pirate(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    builder
        .name("pirate")
        .version(crate::schemas::PKG_VERSION)
        .created_by(crate::schemas::PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldPirateAsset";
    let authoring_schema_func = build_asset_func(fn_name).await?;

    let schema = SchemaSpec::builder()
        .name("pirate")
        .data(
            SchemaSpecData::builder()
                .name("pirate")
                .category("test exclusive")
                .category_name("pirate")
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("pirate_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ff00ff")
                        .func_unique_id(&authoring_schema_func.unique_id)
                        .component_type(ComponentType::Component)
                        .build()
                        .expect("build variant spec data"),
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
        Some(pkg::ImportOptions {
            schemas: Some(vec!["pirate".into()]),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
