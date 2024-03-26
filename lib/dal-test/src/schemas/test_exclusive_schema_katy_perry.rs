use crate::schemas::schema_helpers::{
    build_asset_func, build_codegen_func, build_resource_payload_to_value_func,
    create_identity_func,
};
use dal::pkg::import_pkg_from_pkg;
use dal::{pkg, prop::PropPath, ComponentType};
use dal::{BuiltinsResult, DalContext, PropKind};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, LeafInputLocation, LeafKind, PkgSpec, PropSpec,
    SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg,
};
use si_pkg::{LeafFunctionSpec, SchemaSpecData};

pub async fn migrate_test_exclusive_schema_katy_perry(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut kp_builder = PkgSpec::builder();

    kp_builder
        .name("katy perry")
        .version("2024-03-12")
        .created_by("System Initiative");

    let identity_func_spec = create_identity_func();

    // Create Scaffold Func
    let fn_name = "test:scaffoldKatyPerryAsset";
    let kp_authoring_schema_func = build_asset_func(fn_name).await?;

    // Author Resource Payload Func
    let resource_payload_to_value_func = build_resource_payload_to_value_func().await?;

    // Build YAML CodeGen Func
    let yaml_codegen_fn_name = "test:generateYamlCode";
    let yaml_codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"yaml\",
                    code: Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : \"\"
                };
            }";
    let yaml_code_gen_func =
        build_codegen_func(yaml_codegen_func_code, yaml_codegen_fn_name).await?;

    // Build string CodeGen Func
    let string_codegen_fn_name = "test:generateStringCode";
    let string_codegen_func_code = "async function main(input: Input): Promise < Output > {
                return {
                    format: \"string\",
                    code: \"poop canoe\"
                };
            }";
    let string_code_gen_func =
        build_codegen_func(string_codegen_func_code, string_codegen_fn_name).await?;

    let kp_schema = SchemaSpec::builder()
        .name("katy perry")
        .data(
            SchemaSpecData::builder()
                .name("katy perry")
                .category("test exclusive")
                .category_name("katy perry")
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("katy_perry_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&kp_authoring_schema_func.unique_id)
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
                .leaf_function(
                    LeafFunctionSpec::builder()
                        .func_unique_id(yaml_codegen_fn_name)
                        .leaf_kind(LeafKind::CodeGeneration)
                        .inputs(vec![LeafInputLocation::Domain])
                        .build()?,
                )
                .leaf_function(
                    LeafFunctionSpec::builder()
                        .func_unique_id(string_codegen_fn_name)
                        .leaf_kind(LeafKind::CodeGeneration)
                        .inputs(vec![LeafInputLocation::Domain])
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let kp_spec = kp_builder
        .func(identity_func_spec)
        .func(resource_payload_to_value_func)
        .func(yaml_code_gen_func)
        .func(string_code_gen_func)
        .func(kp_authoring_schema_func)
        .schema(kp_schema)
        .build()?;

    let kp_pkg = SiPkg::load_from_spec(kp_spec)?;
    import_pkg_from_pkg(
        ctx,
        &kp_pkg,
        Some(pkg::ImportOptions {
            schemas: Some(vec!["katy perry".into()]),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
