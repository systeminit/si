use crate::schemas::schema_helpers::{build_asset_func, create_identity_func};
use dal::pkg::import_pkg_from_pkg;
use dal::{prop::PropPath, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, PkgSpec, PropSpec, SchemaSpec, SchemaVariantSpec,
    SchemaVariantSpecData, SiPkg,
};
use si_pkg::{SchemaSpecData, SocketSpec, SocketSpecData, SocketSpecKind};

const CATEGORY: &str = "validations";

pub(crate) async fn migrate_test_exclusive_schema_bad_validations(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "BadValidations";

    builder
        .name(schema_name)
        .version("2024-03-12")
        .created_by("System Initiative");

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = format!("test:scaffold{schema_name}Asset");
    let authoring_schema_func = build_asset_func(fn_name.as_str()).await?;

    let schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(CATEGORY)
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
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
                        .name("good_validations")
                        .kind(PropKind::Integer)
                        .validation_format(r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":0}},{"name":"max","args":{"limit":2}}]}"#)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("bad_validation_format")
                        .kind(PropKind::Integer)
                        .validation_format("5") // Valid Json, bad format
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("bad_validation_json")
                        .kind(PropKind::Integer)
                        .validation_format("'{}'") // invalid Json
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
    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_validated_output(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "ValidatedOutput";

    builder
        .name(schema_name)
        .version("2024-03-12")
        .created_by("System Initiative");

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = format!("test:scaffold{schema_name}Asset");
    let authoring_schema_func = build_asset_func(fn_name.as_str()).await?;

    let schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(CATEGORY)
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
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
                        .name("a_number")
                        .kind(PropKind::Integer)
                        .validation_format(r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":0}},{"name":"max","args":{"limit":2}}]}"#)
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("number")
                        .data(
                            SocketSpecData::builder()
                                .name("number")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "number",
                                ])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "domain", "a_number"]))
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
    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_validated_input(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "ValidatedInput";

    builder
        .name(schema_name)
        .version("2024-03-12")
        .created_by("System Initiative");

    let identity_func_spec = create_identity_func()?;

    // Create Scaffold Func
    let fn_name = format!("test:scaffold{schema_name}Asset");
    let authoring_schema_func = build_asset_func(fn_name.as_str()).await?;

    let schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(CATEGORY)
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
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
                        .name("a_number")
                        .kind(PropKind::Integer)
                        .validation_format(r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":1}},{"name":"max","args":{"limit":2}}]}"#)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .type_prop(
                            PropSpec::builder()
                                .kind(PropKind::Integer)
                                .name("a_number")
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("number")
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("number")
                        .data(
                            SocketSpecData::builder()
                                .name("number")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "number",
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
    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(())
}
